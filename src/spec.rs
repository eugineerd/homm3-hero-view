use crate::utils::RawImage;

#[derive(Clone)]
pub struct Spec {
    pub name: String,
    pub image: RawImage,
}
