use eframe::{egui, epi};
use image::GenericImageView;

pub const H_GOLD: egui::Color32 = egui::Color32::from_rgb(248, 230, 194);

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawImage {
    pub texture_id: Option<egui::TextureId>,
    pub bytes: Box<Vec<u8>>,
    pub dimensions: (f32, f32),
}

impl RawImage {
    pub fn from_bytes(bytes: &[u8], frame: &mut epi::Frame<'_>) -> RawImage {
        let static_image_bytes = bytes;
        let static_image = image::load_from_memory(static_image_bytes).unwrap();
        let size = (
            static_image.width() as usize,
            static_image.height() as usize,
        );

        let pixels: Vec<_> = static_image
            .to_rgba8()
            .chunks_exact(4)
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        let texture = frame
            .tex_allocator()
            .alloc_srgba_premultiplied(size, &pixels);

        let bytes = Box::new(Vec::from_iter(bytes.iter().cloned()));

        RawImage {
            texture_id: Some(texture),
            bytes,
            dimensions: (size.0 as f32, size.1 as f32),
        }
    }

    pub fn load_bytes(&mut self, bytes: &[u8], frame: &mut epi::Frame<'_>) {
        let new_image = RawImage::from_bytes(bytes, frame);
        self.dimensions = new_image.dimensions;
        self.bytes = new_image.bytes;
        self.texture_id = new_image.texture_id;
    }

    pub fn image(&self) -> egui::Image {
        egui::Image::new(self.texture_id.unwrap_or_default(), self.dimensions)
    }

    pub fn image_button(&self) -> egui::ImageButton {
        egui::ImageButton::new(self.texture_id.unwrap_or_default(), self.dimensions)
    }
}

pub fn selected_frame_around(ui: &mut egui::Ui, mut rect: egui::Rect) {
    rect = rect.expand(1.0);
    ui.painter()
        .rect_stroke(rect, 0.0, egui::Stroke::new(1., H_GOLD));
}
