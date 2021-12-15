use crate::utils::RawImage;

#[derive(Clone)]
pub struct Skill {
    pub name: String,
    pub level: u8,
    pub image: RawImage,
}

pub fn demo_skills(frame: &mut eframe::epi::Frame<'_>) -> Vec<Skill> {
    vec![
        Skill {
            name: "Некромантия".to_string(),
            level: 3,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/necro_3.png"), frame),
        },
        Skill {
            name: "Торговля".to_string(),
            level: 1,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/merchant_1.png"), frame),
        },
        Skill {
            name: "Нападение".to_string(),
            level: 1,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/offence_1.png"), frame),
        },
        Skill {
            name: "Удача".to_string(),
            level: 1,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/luck_1.png"), frame),
        },
        Skill {
            name: "Удача".to_string(),
            level: 2,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/luck_2.png"), frame),
        },
        Skill {
            name: "Удача".to_string(),
            level: 3,
            image: RawImage::from_bytes(include_bytes!("../resources/skill/luck_3.png"), frame),
        },
    ]
}
