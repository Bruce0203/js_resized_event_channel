use winit::dpi::PhysicalSize;

///```rust
///fn test() {
/// use js_resized_event_channel::{JsResizeEventChannel, ResizeEventChannel};
/// use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowAttributes};
///
/// fn main() {
///     let event_loop: EventLoop<()> = EventLoop::new().unwrap();
///     let window = event_loop
///         .create_window(WindowAttributes::default())
///         .unwrap();
///     let resize_event_channel = JsResizeEventChannel::init(&window);
///     event_loop
///         .run(|event, event_loop| {
///             if resize_event_channel.try_recv_resized_event() {
///                 let _ = window.request_inner_size(resize_event_channel.size());
///             }
///         })
///         .unwrap();
/// }
///}
///```
pub struct JsResizeEventChannel {
    #[cfg(target_arch = "wasm32")]
    receiver: kanal::AsyncReceiver<()>,
}

pub trait ResizeEventChannel {
    fn init(window: &winit::window::Window) -> Self;
    fn try_recv_resized_event(&self) -> bool;
    fn size(&self) -> PhysicalSize<u32>;
}

#[cfg(not(target_arch = "wasm32"))]
impl ResizeEventChannel for JsResizeEventChannel {
    fn init(_window: &winit::window::Window) -> Self {
        JsResizeEventChannel {}
    }

    fn try_recv_resized_event(&self) -> bool {
        false
    }

    fn size(&self) -> PhysicalSize<u32> {
        unreachable!()
    }
}

#[cfg(target_arch = "wasm32")]
impl ResizeEventChannel for JsResizeEventChannel {
    fn init(window: &winit::window::Window) -> Self {
        let (sender, receiver) = kanal::unbounded_async();
        Self::setup_canvas(window);
        Self::register_resize_event_to_js(sender);
        Self { receiver }
    }

    fn try_recv_resized_event(&self) -> bool {
        if let Ok(Some(())) = self.receiver.try_recv() {
            true
        } else {
            false
        }
    }

    fn size(&self) -> PhysicalSize<u32> {
        Self::size_of_window()
    }
}

#[cfg(target_arch = "wasm32")]
impl JsResizeEventChannel {
    fn setup_canvas(window: &winit::window::Window) {
        let canvas = winit::platform::web::WindowExtWebSys::canvas(window).unwrap();
        let document = web_sys::window().unwrap().document().unwrap();
        let screen = document.get_element_by_id("screen").unwrap();
        screen.append_child(&canvas).unwrap();
        let _ = window.request_inner_size(Self::size_of_window());
    }

    fn register_resize_event_to_js(sender: kanal::AsyncSender<()>) {
        let f = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
            pollster::block_on(sender.send(())).unwrap();
        }) as Box<dyn FnMut()>);
        let window = web_sys::window().unwrap();
        window.set_onresize(Some(wasm_bindgen::JsCast::unchecked_ref(f.as_ref())));
        f.forget();
    }

    fn size_of_window() -> PhysicalSize<u32> {
        let window = web_sys::window().unwrap();
        PhysicalSize::new(
            window.inner_width().unwrap().as_f64().unwrap() as u32 * 2,
            window.inner_height().unwrap().as_f64().unwrap() as u32 * 2,
        )
    }
}
