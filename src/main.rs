extern crate gio;
extern crate glib;
extern crate gstreamer as gst;
extern crate gtk;

use gio::prelude::*;
use gst::prelude::*;
use gst::{Element,};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, };

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("window")
            .expect("Missing/corrupted Glade file");

    window.set_application(application);

    window.connect_delete_event(move |win, _| {
        win.destroy();
        Inhibit(false)
    });

    window.show_all();

    /*
    let pipeline_def = "wasapisrc low-latency=true ! audio/x-raw,rate=96000,channels=32 ! \
         audioconvert mix-matrix=\"<<(float)1.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0,  \
        (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0, (float)0.0>>\" ";
    */
    let mut pipeline_def: String = String::from("pulsesrc ");
    pipeline_def.push_str("! audio/x-raw,channels=1 ! audioresample ! audio/x-raw,rate=48000 ! opusenc ! \
        rtpopuspay ! udpsink port=12345");

    let pipeline = gst::parse_launch(&pipeline_def).unwrap();
    let pipeline_weak = pipeline.downgrade();



    let timeout_id = gtk::timeout_add(100 /*ms*/, move || {
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None => return glib::Continue(true),
        };

        let ret = pipeline.set_state(gst::State::Playing);
        //pipeline.get_state()
        assert_ne!(ret, gst::StateChangeReturn::Failure);
        glib::Continue(true)
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
