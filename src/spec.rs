use crate::utils::RawImage;

#[derive(Clone, Debug, Default)]
pub struct Spec {
    pub name: String,
    pub class: String,
    pub image: RawImage,
}

impl PartialEq for Spec {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
