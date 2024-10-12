#![cfg(target_arch = "wasm32")]

use kanal::{AsyncReceiver, AsyncSender};
use web_sys::{Element, HtmlCanvasElement};
use winit::dpi::PhysicalSize;

pub struct JsResizeEventChannel {
    receiver: AsyncReceiver<()>,
    canvas: HtmlCanvasElement,
}

impl JsResizeEventChannel {
    pub fn init(window: &winit::window::Window, dst: Element) -> Self {
        let (sender, receiver) = kanal::unbounded_async();
        Self::register_resize_event_to_js(sender);
        let canvas = Self::setup_canvas(window, dst);
        Self { receiver, canvas }
    }

    pub fn try_recv_resized_event(&self) -> bool {
        if let Ok(Some(())) = self.receiver.try_recv() {
            true
        } else {
            false
        }
    }

    pub fn try_resize_event(&self, window: &winit::window::Window) {
        if self.try_recv_resized_event() {
            let _ = window.request_inner_size(self.size());
        }
    }

    fn setup_canvas(window: &winit::window::Window, dst: Element) -> HtmlCanvasElement {
        let canvas = winit::platform::web::WindowExtWebSys::canvas(window).unwrap();
        dst.append_child(&canvas).expect("Cannot append canvas");

        let _ = window.request_inner_size(size_of_window());
        canvas
    }

    fn register_resize_event_to_js(canvas: &HtmlCanvasElement, sender: AsyncSender<()>) {
        let f = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
            pollster::block_on(sender.send(())).unwrap();
        }) as Box<dyn FnMut()>);
        canvas.set_onresize(Some(wasm_bindgen::JsCast::unchecked_ref(f.as_ref())));
        f.forget();
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        size_of_canvas(&self.canvas)
    }
}

fn size_of_canvas(canvas: &HtmlCanvasElement) -> PhysicalSize<u32> {
    PhysicalSize::new(
        canvas.scroll_width() as u32 * 2,
        canvas.scroll_height() as u32 * 2,
    )
}

fn size_of_window() -> PhysicalSize<u32> {
    let window = web_sys::window().unwrap();
    PhysicalSize::new(
        window.inner_width().unwrap().as_f64().unwrap() as u32 * 2,
        window.inner_height().unwrap().as_f64().unwrap() as u32 * 2,
    )
}
