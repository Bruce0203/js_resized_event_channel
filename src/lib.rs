use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    js_sys::Array, HtmlElement, ResizeObserver, ResizeObserverEntry, ResizeObserverSize,
};
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
///             if let Some(size) = resize_event_channel.try_recv_resized_event() {
///                 let _ = window.request_inner_size(size);
///             }
///         })
///         .unwrap();
/// }
///}
///```
pub struct JsResizeEventChannel {
    #[cfg(target_arch = "wasm32")]
    receiver: kanal::AsyncReceiver<PhysicalSize<u32>>,
}

pub trait ResizeEventChannel {
    fn init(window: &winit::window::Window) -> Self;
    fn try_recv_resized_event(&self) -> Option<PhysicalSize<u32>>;
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

    fn try_recv_resized_event(&self) -> Option<PhysicalSize<u32>> {
        if let Ok(Some(size)) = self.receiver.try_recv() {
            Some(size)
        } else {
            None
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl JsResizeEventChannel {
    fn setup_canvas(window: &winit::window::Window) {
        let canvas = winit::platform::web::WindowExtWebSys::canvas(window).unwrap();
        Self::get_element_of_screen().append_child(&canvas).unwrap();
        let _ = window.request_inner_size(Self::size_of_window());
    }

    fn register_resize_event_to_js(sender: kanal::AsyncSender<PhysicalSize<u32>>) {
        let on_resize = Closure::<dyn FnMut(Array)>::new(move |entries: Array| {
            let entry = entries.at(0);
            let entry: ResizeObserverEntry = entry.dyn_into().unwrap();
            let size: ResizeObserverSize = entry.content_box_size().at(0).dyn_into().unwrap();
            let mut size = PhysicalSize::new(size.inline_size() as u32, size.block_size() as u32);
            size.width = size.width / 16;
            size.height = size.height / 16;
            let canvas = entry.target();
            let width = size.width.to_string();
            let height = size.height.to_string();
            canvas.set_attribute("width", width.as_str()).unwrap();
            canvas.set_attribute("height", height.as_str()).unwrap();
            pollster::block_on(sender.send(size)).unwrap();
        });
        let resize_observer = ResizeObserver::new(on_resize.as_ref().unchecked_ref()).unwrap();
        resize_observer.observe(&Self::get_element_of_screen());
        on_resize.forget();
    }

    fn get_element_of_screen() -> web_sys::Element {
        let document = web_sys::window().unwrap().document().unwrap();
        document.get_element_by_id("screen").unwrap()
    }

    fn size_of_window() -> PhysicalSize<u32> {
        let window = web_sys::window().unwrap();
        PhysicalSize::new(
            window.inner_width().unwrap().as_f64().unwrap() as u32 * 2,
            window.inner_height().unwrap().as_f64().unwrap() as u32 * 2,
        )
    }
}
