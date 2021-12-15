mod backend;
mod backpack;
mod character;
mod edit_views;
mod geometry;
mod hero;
mod hero_viewer;
mod skill;
mod spec;
mod unit;
mod utils;

pub use backend::DemoBackend;
pub use geometry::WINDOW_SIZE;
pub use hero_viewer::HeroViewer;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
    let app = self::HeroViewer::<DemoBackend>::default();
    eframe::start_web(canvas_id, Box::new(app))
}
