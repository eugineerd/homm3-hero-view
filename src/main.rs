use homm3_hero_viewer::{HeroViewer, WINDOW_SIZE};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use homm3_hero_viewer::DemoBackend;

    let hero_viewer = Box::new(HeroViewer::<DemoBackend>::default());
    let options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(WINDOW_SIZE),
        ..Default::default()
    };

    eframe::run_native(hero_viewer, options);
}
