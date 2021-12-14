use crate::backpack::Backpack;
use crate::character::Character;
use crate::skill::Skill;
use crate::spec::Spec;
use crate::unit::Unit;
use crate::utils::RawImage;
pub struct Hero {
    pub id: usize,
    pub character: Character,
    pub units: Vec<Unit>,
    pub skills: Vec<Skill>,
    pub backpack: Backpack,

    pub pskills: [u8; 4],
    pub experience: u16,
    pub mana_max: u16,
    pub mana_current: u16,
    pub level: u8,
    pub spec: Spec,
}

pub fn demo_heroes(frame: &mut eframe::epi::Frame<'_>) -> Vec<Hero> {
    let mut heroes = Vec::new();

    let character1 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl004pl.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps004pl.png"), frame),
        name: "Монер".to_string(),
        class: "Путешественник".to_string(),
    };

    let units1 = Vec::new();
    let skills1 = Vec::new();
    let backpack1 = Backpack {};

    let hero1 = Hero {
        id: 0,
        character: character1,
        units: units1,
        skills: skills1,
        backpack: backpack1,

        pskills: [10, 4, 12, 5],
        experience: 37000,
        mana_max: 334,
        mana_current: 210,
        level: 17,
        spec: Spec {
            name: "Элементали".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/00_128.png"), frame),
        },
    };

    heroes.push(hero1);

    let character2 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl033al.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps033al.png"), frame),
        name: "Тан".to_string(),
        class: "Алхимик".to_string(),
    };

    let units2 = Vec::new();
    let skills2 = Vec::new();
    let backpack2 = Backpack {};

    let hero2 = Hero {
        id: 1,
        character: character2,
        units: units2,
        skills: skills2,
        backpack: backpack2,

        pskills: [1, 1, 2, 2],
        experience: 0,
        mana_max: 10,
        mana_current: 10,
        level: 1,
        spec: Spec {
            name: "Джинны".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/00_33.png"), frame),
        },
    };

    heroes.push(hero2);

    heroes
}
