```rust

use js_resized_event_channel::JsResizeEventChannel;
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};

fn main() {
    let web_window = web_sys::window().unwrap();
    let event_loop: EventLoop<()> = EventLoop::builder().build().unwrap();
    let window = event_loop
        .create_window(WindowAttributes::default())
        .unwrap();
    let resize_event_channel = JsResizeEventChannel::init(
        &window,
        web_window
            .document()
            .unwrap()
            .get_element_by_id("container")
            .unwrap(),
    );
    event_loop
        .run(|_, _| {
            if resize_event_channel.try_recv_resized_event() {
                let _ = window.request_inner_size(PhysicalSize {
                    width: web_window.inner_width().unwrap().as_f64().unwrap() as u32 * 2,
                    height: web_window.inner_height().unwrap().as_f64().unwrap() as u32 * 2,
                });
            }
        })
        .unwrap();
}
```
