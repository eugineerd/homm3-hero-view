use crate::backpack::Backpack;
use crate::character::Character;
use crate::skill::Skill;
use crate::spec::Spec;
use crate::unit::Unit;
use crate::utils::RawImage;
pub struct Hero {
    pub id: usize,
    pub character: Character,
    pub units: [Option<Unit>; 7],
    pub skills: [Option<Skill>; 8],
    pub backpack: Backpack,

    pub pskills: [u8; 4],
    pub spec: Spec,
    pub luck: u8,
    pub morale: u8,
    pub experience: u16,
    pub mana_max: u16,
    pub mana_current: u16,
    pub level: u8,
}

pub fn demo_heroes(frame: &mut eframe::epi::Frame<'_>) -> Vec<Hero> {
    let mut heroes = Vec::new();

    let character1 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl004pl.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps004pl.png"), frame),
        name: "Монер".to_string(),
        class: "Путешественник".to_string(),
    };

    let units1 = Default::default();
    let mut skills1: [Option<Skill>; 8] = Default::default();
    skills1[0] = Some(Skill {
        image: RawImage::from_bytes(include_bytes!("../resources/skill/necro_3.png"), frame),
        name: "Некромантия".to_string(),
        level: 3,
    });
    skills1[1] = Some(Skill {
        image: RawImage::from_bytes(include_bytes!("../resources/skill/offence_1.png"), frame),
        name: "Нападение".to_string(),
        level: 1,
    });
    let backpack1 = Backpack {};

    let hero1 = Hero {
        id: 0,
        character: character1,
        units: units1,
        skills: skills1,
        backpack: backpack1,

        pskills: [10, 4, 12, 5],
        experience: 37000,
        luck: 6,
        morale: 1,
        mana_max: 334,
        mana_current: 210,
        level: 17,
        spec: Spec {
            name: "Элементали".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_psych.png"), frame),
        },
    };

    heroes.push(hero1);

    let character2 = Character {
        portrait: RawImage::from_bytes(include_bytes!("../resources/hpl033al.png"), frame),
        portrait_small: RawImage::from_bytes(include_bytes!("../resources/hps033al.png"), frame),
        name: "Тан".to_string(),
        class: "Алхимик".to_string(),
    };

    let units2 = Default::default();
    let skills2 = Default::default();
    let backpack2 = Backpack {};

    let hero2 = Hero {
        id: 1,
        character: character2,
        units: units2,
        skills: skills2,
        backpack: backpack2,

        pskills: [1, 1, 2, 2],
        luck: 1,
        morale: 5,
        experience: 0,
        mana_max: 10,
        mana_current: 10,
        level: 1,
        spec: Spec {
            name: "Джинны".to_string(),
            image: RawImage::from_bytes(include_bytes!("../resources/spec_djini.png"), frame),
        },
    };

    heroes.push(hero2);

    heroes
}
