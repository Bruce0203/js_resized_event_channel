use winit::dpi::PhysicalSize;

type Size = PhysicalSize<f64>;

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
    receiver: kanal::AsyncReceiver<Size>,
}

pub trait ResizeEventChannel {
    fn init(window: &winit::window::Window) -> Self;
    fn try_recv_resized_event(&self) -> Option<Size>;
}

#[cfg(not(target_arch = "wasm32"))]
impl ResizeEventChannel for JsResizeEventChannel {
    fn init(_window: &winit::window::Window) -> Self {
        JsResizeEventChannel {}
    }

    fn try_recv_resized_event(&self) -> Option<PhysicalSize<f64>> {
        None
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

    fn try_recv_resized_event(&self) -> Option<Size> {
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
    }

    fn register_resize_event_to_js(sender: kanal::AsyncSender<Size>) {
        let on_resize = wasm_bindgen::prelude::Closure::<dyn FnMut(web_sys::js_sys::Array)>::new(
            move |entries: web_sys::js_sys::Array| {
                let entry = entries.at(0);
                let entry: web_sys::ResizeObserverEntry =
                    wasm_bindgen::JsCast::dyn_into(entry).unwrap();
                let size: web_sys::ResizeObserverSize =
                    wasm_bindgen::JsCast::dyn_into(entry.content_box_size().at(0)).unwrap();
                let mut size = PhysicalSize::new(size.inline_size(), size.block_size());
                size.width *= 2.;
                size.height *= 2.;
                let canvas = entry.target();
                let width = size.width.to_string();
                let height = size.height.to_string();
                pollster::block_on(sender.send(size)).unwrap();
            },
        );
        let resize_observer =
            web_sys::ResizeObserver::new(wasm_bindgen::JsCast::unchecked_ref(on_resize.as_ref()))
                .unwrap();
        resize_observer.observe(&Self::get_element_of_screen());
        on_resize.forget();
    }

    fn get_element_of_screen() -> web_sys::Element {
        let document = web_sys::window().unwrap().document().unwrap();
        document.get_element_by_id("screen").unwrap()
    }
}
