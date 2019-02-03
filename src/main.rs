extern crate gio;
extern crate glib;
extern crate gstreamer as gst;
extern crate gtk;

use gio::prelude::*;
use gst::prelude::*;
use gst::{Element,BinExt};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Builder, };

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("main.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("window")
            .expect("Missing/corrupted Glade file");

    let btnRecord: gtk::Button = builder.get_object("btnRecord1").expect("Couldn't find btnRecord1");

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
    pipeline_def.push_str("! audio/x-raw,channels=1 ! audioresample ! audio/x-raw,rate=48000 ! level post-messages=TRUE ! tee name=t ");
    // Streaming
    pipeline_def.push_str("! queue ! opusenc ! rtpopuspay ! udpsink port=12345 ");
    // Saving to file
    pipeline_def.push_str("t. ! queue ! opusenc ! oggmux ! filesink location=out.opus");

    //let pipeline_def = "pulsesrc ! level post-messages=TRUE ! fakesink sync=TRUE";

    println!("{}", pipeline_def);

    let pipeline = gst::parse_launch(&pipeline_def).unwrap();
    let bus = pipeline.get_bus().unwrap();

    let pipeline_weak = pipeline.downgrade();
    bus.add_watch(move |_, msg| {
        println!("GSTMessage");
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None =>  return glib::Continue(true),
        };
        println!("Woot!");

        glib::Continue(true)
    });

    let pipeline_weak = pipeline.downgrade();
    btnRecord.connect_clicked(move |button| {
        println!("Clicked!");
        /*
        let pipeline = match pipeline_weak.upgrade() {
            Some(pipeline) => pipeline,
            None =>  return,
        };
        */
        println!("Pipeline not deleted!");
        let ret = pipeline.set_state(gst::State::Playing);
        assert_ne!(ret, gst::StateChangeReturn::Failure);
    });

    //let ev = gst::Event::new_eos().build();
    //pipeline.send_event(ev);

    //let level_weak = pipeline.get_by_name("level").unwrap().downgrade();


    //let ret = pipeline.set_state(gst::State::Playing);
    //assert_ne!(ret, gst::StateChangeReturn::Failure);
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
