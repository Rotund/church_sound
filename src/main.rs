extern crate chrono;
extern crate gio;
extern crate glib;
extern crate gstreamer as gst;
extern crate gtk;
extern crate num;

use chrono::prelude::*;
use gio::prelude::*;
use gst::prelude::*;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, };

use std::env::args;
use std::sync::{Arc, Mutex};

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("window")
            .expect("Missing/corrupted Glade file");

    let btn_record: gtk::Button = builder.get_object("btnRecord1").expect("Couldn't find btnRecord1");
    let bar_vu: gtk::ProgressBar = builder.get_object("barVUMeter1").expect("Couldn't find barVUMeter1");
    let vu_db = Arc::new(Mutex::new(0.0));
    let vu_db_read = vu_db.clone();

    window.set_application(application);

    window.connect_delete_event(move |win, _| {
        win.destroy();
        Inhibit(false)
    });

    window.show_all();

    let dt = Local::now();
    let dt_str = dt.format("%Y%m%d%H%M%S").to_string();

    #[cfg(target_os = "windows")]
    let mut pipeline_def = String::from("wasapisrc low-latency=true ! audio/x-raw,rate=96000,channels=32 ! \
         audioconvert mix-matrix=\"<<(float)1.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0>>\" ");

    #[cfg(target_os = "linux")]
    let mut pipeline_def: String = String::from("pulsesrc ");

    pipeline_def.push_str("! audio/x-raw,channels=1 ! audioresample ! audio/x-raw,rate=48000 ! level post-messages=TRUE ! tee name=t ");
    // Streaming
    pipeline_def.push_str("! queue ! opusenc ! rtpopuspay ! udpsink port=12345 ");
    // Saving to file
    pipeline_def.push_str(&format!("t. ! queue ! opusenc ! oggmux ! filesink location=\"f:/recording-{}.opus\"", dt_str));

    println!("{}", pipeline_def);

    let pipeline = gst::parse_launch(&pipeline_def).unwrap();
    let bus = pipeline.get_bus().unwrap();

    let pipeline_weak = pipeline.downgrade();
    bus.add_watch(move |_, msg| {
        let elem_msg = match msg.view() {
            gst::MessageView::Element(elem) => elem,
            _ => return glib::Continue(true),
        };
        let elem_st = elem_msg.get_structure().unwrap();
        if elem_st.get_name() == "level" {
            *vu_db.lock().unwrap() = elem_st.get_value("rms").unwrap().get::<glib::ValueArray>().unwrap()[0].get::<f64>().unwrap();
        }

        glib::Continue(true)
    });

    let pipeline_weak = pipeline.downgrade();
    btn_record.connect_clicked(move |button| {
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None =>  return,
        };
        let ret = pipeline.set_state(gst::State::Playing);
        assert_ne!(ret, gst::StateChangeReturn::Failure);
        button.set_sensitive(false);
    });

    let timeout_id = gtk::timeout_add(100, move || {
        let rms: f64 = *vu_db_read.lock().unwrap();
        bar_vu.set_fraction(num::clamp((rms+60.0)/60.0, 0.0, 1.0));
        glib::Continue(true)
    });



    //let ev = gst::Event::new_eos().build();
    //pipeline.send_event(ev);

    //let level_weak = pipeline.get_by_name("level").unwrap().downgrade();


    //let ret = pipeline.set_state(gst::State::Playing);
    //assert_ne!(ret, gst::StateChangeReturn::Failure);

    // shutdown owns the pipeline
    application.connect_shutdown(move |_| {
        let ret = pipeline.set_state(gst::State::Null);
        assert_ne!(ret, gst::StateChangeReturn::Failure);
    });
}

fn main(){

    // initialize GStreamer and GTK+ and exit if fail
    gst::init().unwrap();
    gtk::init().unwrap();

    let application = gtk::Application::new("org.middletonucc.church_sound",
                                            gio::ApplicationFlags::empty())
                                        .expect("GTK+ Initalization Failed...");


    application.connect_startup(move |app| {
        build_ui(app);
    });

    application.connect_activate(|_| {});
    application.run(&args().collect::<Vec<_>>());



/*
    let ret = pipeline.set_state(gst::State::Playing);
    assert_ne!(ret, gst::StateChangeReturn::Failure);

    let bus = pipeline.get_bus().unwrap();
    while let Some(msg) = bus.timed_pop(gst::CLOCK_TIME_NONE)
    {
        use gst::MessageView;

        match msg.view()
        {
            MessageView::Eos(..) => break,
            MessageView::Error(err) =>
            {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.get_src().map(|s| s.get_path_string()),
                    err.get_error(),
                    err.get_debug()
                );
                break;
            }
            _ => (),
        }
    }
    let ret = pipeline.set_state(gst::State::Null);
    assert_ne!(ret, gst::StateChangeReturn::Failure);
*/
}
