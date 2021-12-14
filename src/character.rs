use crate::utils::RawImage;

pub struct Character {
    pub portrait: RawImage,
    pub portrait_small: RawImage,
    pub name: String,
    pub class: String,
}
