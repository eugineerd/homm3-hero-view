use crate::utils::RawImage;

#[derive(Clone, Default)]
pub struct Character {
    pub portrait: RawImage,
    pub name: String,
    pub class: String,
}
