```rust
 use js_resized_event_channel::{JsResizeEventChannel, ResizeEventChannel};
 use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};

 fn main() {
     let event_loop: EventLoop<()> = EventLoop::new().unwrap();
     let window = event_loop
         .create_window(WindowAttributes::default())
         .unwrap();
     let resize_event_channel = JsResizeEventChannel::init(&window);
     event_loop
         .run(|event, event_loop| {
             if resize_event_channel.try_recv_resized_event() {
                 let _ = window.request_inner_size(resize_event_channel.size());
             }
         })
         .unwrap();
 }

```
