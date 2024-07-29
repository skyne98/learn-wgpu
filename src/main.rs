pub mod application;
pub mod logging;
pub mod model;
pub mod texture;

use application::ApplicationFlow;
use std::borrow::Cow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    }
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    #[allow(unused_mut)]
    let mut window_attr = winit::window::Window::default_attributes();
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        // check if we can get the canvas element
        let canvas = web_sys::window()
            .expect("Failed to get window")
            .document()
            .expect("Failed to get document")
            .get_element_by_id("canvas");
        // if not found, create a new canvas element
        let canvas = match canvas {
            Some(canvas) => canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok(),
            None => {
                let canvas = web_sys::window()
                    .expect("Failed to get window")
                    .document()
                    .expect("Failed to get document")
                    .create_element("canvas")
                    .expect("Failed to create canvas")
                    .dyn_into::<web_sys::HtmlCanvasElement>()
                    .expect("Failed to convert to HtmlCanvasElement");
                canvas.set_id("canvas");
                // make it fullscreen using css
                canvas.style().set_property("position", "absolute").unwrap();
                canvas.style().set_property("top", "0").unwrap();
                canvas.style().set_property("left", "0").unwrap();
                canvas.style().set_property("width", "100%").unwrap();
                canvas.style().set_property("height", "100%").unwrap();
                web_sys::window()
                    .expect("Failed to get window")
                    .document()
                    .expect("Failed to get document")
                    .body()
                    .expect("Failed to get body")
                    .append_child(&canvas)
                    .expect("Failed to append canvas to body");
                Some(canvas)
            }
        };
        let canvas = canvas.expect("Failed to get canvas");
        window_attr = window_attr.with_canvas(Some(canvas));
    }

    logging::setup_tracing();
    let window = event_loop
        .create_window(window_attr)
        .expect("Failed to create window");
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut application = pollster::block_on(ApplicationFlow::new(&window));
        let _ = event_loop.run_app(&mut application);
    }
    #[cfg(target_arch = "wasm32")]
    {
        let future = async {
            let window = window;
            let mut application = ApplicationFlow::new(&window).await;
            let _ = event_loop.run_app(&mut application);
        };
        wasm_bindgen_futures::spawn_local(future);
    }
}
