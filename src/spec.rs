use crate::utils::RawImage;

#[derive(Clone, Debug, Default)]
pub struct Spec {
    pub name: String,
    pub class: String,
    pub image: RawImage,
}

impl PartialEq for Spec {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub fn demo_specs(frame: &mut eframe::epi::Frame<'_>) -> Vec<Spec> {
    vec![
        Spec {
            name: "Ускорение".to_string(),
            class: "Путешественник".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_speed.png"), frame),
        },
        Spec {
            name: "Волшебство".to_string(),
            class: "Путешественник".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_wizard.png"), frame),
        },
        Spec {
            name: "Элементали".to_string(),
            class: "Путешественник".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_psych.png"), frame),
        },
        Spec {
            name: "Джинны".to_string(),
            class: "Алхимик".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_djini.png"), frame),
        },
    ]
}
