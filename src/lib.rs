use cfg_if::cfg_if;
use kanal::{AsyncReceiver, AsyncSender};
use winit::dpi::PhysicalSize;

pub struct JsResizeEventChannel {
    #[cfg(target_arch = "wasm32")]
    receiver: AsyncReceiver<()>,
}

macro_rules! to_f64_from_js_value {
    ($size:expr) => {
        $size.unwrap().as_ref().as_f64().unwrap() as u32
    };
}

impl JsResizeEventChannel {
    pub fn init(window: &winit::window::Window) -> Self {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let (sender, receiver) = kanal::unbounded_async();
                Self::register_resize_event_from_js(sender);
                Self::setup_canvas(window);
                Self { receiver }
            } else {
                Self {}
            }
        }
    }

    pub fn try_resize_event(&self, window: &winit::window::Window) {
        if let Some(size) = self.try_recv_resized_event() {
            let _ = window.request_inner_size(size);
        }
    }

    fn setup_canvas(window: &winit::window::Window) {
        #[cfg(target_arch = "wasm32")]
        {
            let canvas = winit::platform::web::WindowExtWebSys::canvas(window).unwrap();
            web_sys::window()
                .and_then(|win| {
                    let doc = win.document()?;
                    let dst = doc.get_element_by_id("container")?;
                    dst.append_child(&canvas).ok()?;
                    std::hint::black_box(window.request_inner_size(winit::dpi::PhysicalSize::new(
                        to_f64_from_js_value!(win.inner_width()),
                        to_f64_from_js_value!(win.inner_height()),
                    )));
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }
    }

    fn register_resize_event_from_js(sender: AsyncSender<()>) {
        #[cfg(target_arch = "wasm32")]
        {
            let f = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
                pollster::block_on(sender.send(())).unwrap();
            }) as Box<dyn FnMut()>);
            web_sys::window()
                .unwrap()
                .set_onresize(Some(wasm_bindgen::JsCast::unchecked_ref(f.as_ref())));
            f.forget();
        }
    }

    fn try_recv_resized_event(&self) -> Option<PhysicalSize<u32>> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                if let Ok(Some(())) = self.receiver.try_recv() {
                    let window = web_sys::window().unwrap();
                    let size = PhysicalSize::new(
                        to_f64_from_js_value!(window.inner_width()) * 2,
                        to_f64_from_js_value!(window.inner_height()) * 2,
                    );
                    Some(size)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}
