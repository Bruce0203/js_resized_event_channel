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

    pub fn try_recv_resized_event(&self) -> Option<PhysicalSize<u32>> {
        if let Ok(Some(())) = self.receiver.try_recv() {
            let size = PhysicalSize::new(
                self.canvas.scroll_width() as u32,
                self.canvas.scroll_height() as u32,
            );
            Some(size)
        } else {
            None
        }
    }

    pub fn try_resize_event(&self, window: &winit::window::Window) {
        if let Some(size) = self.try_recv_resized_event() {
            let _ = window.request_inner_size(size);
        }
    }

    fn setup_canvas(window: &winit::window::Window, dst: Element) -> HtmlCanvasElement {
        let canvas = winit::platform::web::WindowExtWebSys::canvas(window).unwrap();
        dst.append_child(&canvas).expect("Cannot append canvas");

        std::hint::black_box(window.request_inner_size(winit::dpi::PhysicalSize::new(
            dst.scroll_width(),
            dst.scroll_height(),
        )));
        canvas
    }

    fn register_resize_event_to_js(sender: AsyncSender<()>) {
        let f = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
            pollster::block_on(sender.send(())).unwrap();
        }) as Box<dyn FnMut()>);
        web_sys::window()
            .unwrap()
            .set_onresize(Some(wasm_bindgen::JsCast::unchecked_ref(f.as_ref())));
        f.forget();
    }
}
