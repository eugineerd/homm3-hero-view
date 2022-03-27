use crate::utils::RawImage;

#[derive(Clone, Default)]
pub struct Skill {
    pub id: usize,
    pub name: String,
    pub level: u8,
    pub image: RawImage,
}

impl PartialEq for Skill {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
