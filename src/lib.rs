use cfg_if::cfg_if;
use kanal::{AsyncReceiver, AsyncSender};
use winit::dpi::PhysicalSize;

pub struct JsResizeEventChannel {
    #[cfg(target_arch = "wasm32")]
    receiver: AsyncReceiver<()>,
}

impl JsResizeEventChannel {
    pub fn init() -> Self {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let (sender, receiver) = kanal::unbounded_async();
                Self::register_resize_event_from_js(sender);
                Self { receiver }
            } else {
                Self {}
            }
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

    pub fn try_recv_resized_event(&self) -> Option<PhysicalSize<u32>> {
        #[cfg(target_arch = "wasm32")]
        if let Ok(Some(())) = self.receiver.try_recv() {
            let window = web_sys::window().unwrap();
            macro_rules! to_f64_from_js_value {
                ($size:expr) => {
                    $size.unwrap().as_ref().as_f64().unwrap()
                };
            }
            let size = PhysicalSize::new(
                to_f64_from_js_value!(window.inner_width()) as u32 * 2,
                to_f64_from_js_value!(window.inner_height()) as u32 * 2,
            );
            Some(size)
        } else {
            None
        }
    }
}
