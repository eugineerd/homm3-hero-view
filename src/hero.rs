use crate::character::Character;
use crate::skill::Skill;
use crate::spec::Spec;
use crate::utils::RawImage;

#[derive(Clone, Default)]
pub struct Hero {
    pub id: usize,
    pub character: Character,

    pub skills: [Option<Skill>; 8],
    pub pskills: [u8; 4],
    pub spec: Spec,
    pub luck: u8,
    pub morale: u8,
    pub experience: u16,
    pub mana_max: u16,
    pub mana_current: u16,
    pub level: u8,
}

#[derive(Clone, Default)]
pub struct HeroSelectButton {
    pub id: usize,
    pub portrait: RawImage,
}
