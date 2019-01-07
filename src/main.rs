extern crate gstreamer as gst;

use gst::GstObjectExt;
use gst::ElementExt;

fn main()
{
    println!("Hello, world!");

    // initialize GStreamer and exit if fail
    gst::init().unwrap();

    let pipeline = gst::parse_launch(&"pulsesrc ! opusenc ! rtpopuspay ! udpsink port=12345").unwrap();

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
}
