use crate::utils::RawImage;

#[derive(Clone, Debug)]
pub struct Character {
    pub portrait: RawImage,
    pub portrait_small: RawImage,
    pub name: String,
    pub class: String,
}
