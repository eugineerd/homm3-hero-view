use crate::utils::RawImage;
pub struct Hero {
    pub character: Character,

    pub attack: u8,
    pub defense: u8,
    pub mpower: u8,
    pub knowledge: u8,
    pub experience: u16,
    pub mana_max: u16,
    pub mana_current: u16,
    pub level: u8,
    pub specialty_name: String,
    pub specialty_image: RawImage,
}

pub struct Character {
    pub portrait: RawImage,
    pub portrait_small: RawImage,
    pub name: String,
    pub class: String,
}

pub fn demo_heroes(frame: &mut eframe::epi::Frame<'_>) -> Vec<Hero> {
    let mut heroes = Vec::new();

    let character1 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl004pl.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps004pl.png"), frame),
        name: "Монер".to_string(),
        class: "Путешественник".to_string(),
    };

    let hero1 = Hero {
        character: character1,
        attack: 10,
        defense: 4,
        mpower: 12,
        knowledge: 5,
        experience: 37000,
        mana_max: 334,
        mana_current: 210,
        level: 17,
        specialty_name: "Элементали".to_string(),
        specialty_image: RawImage::from_bytes(include_bytes!("../resources/00_128.png"), frame),
    };

    heroes.push(hero1);

    let character2 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl033al.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps033al.png"), frame),
        name: "Тан".to_string(),
        class: "Алхимик".to_string(),
    };

    let hero2 = Hero {
        character: character2,
        attack: 1,
        defense: 1,
        mpower: 2,
        knowledge: 2,
        experience: 0,
        mana_max: 10,
        mana_current: 10,
        level: 1,
        specialty_name: "Джинны".to_string(),
        specialty_image: RawImage::from_bytes(include_bytes!("../resources/00_33.png"), frame),
    };

    heroes.push(hero2);

    heroes
}
