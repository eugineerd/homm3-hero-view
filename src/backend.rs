use crate::skill::{demo_skills, Skill};
use crate::spec::{demo_specs, Spec};

pub trait Backend {
    fn update(&mut self, frame: &mut eframe::epi::Frame<'_>);
    fn get_specs(&mut self, class: &str) -> &[Spec];
    fn get_skills(&mut self) -> &[Skill];
}

#[derive(Default)]
pub struct DemoBackend {
    specs: Vec<Spec>,
    skills: Vec<Skill>,
}

impl Backend for DemoBackend {
    fn get_specs(&mut self, class: &str) -> &[Spec] {
        &self.specs
    }
    fn get_skills(&mut self) -> &[Skill] {
        &self.skills
    }
    fn update(&mut self, frame: &mut eframe::epi::Frame<'_>) {
        if self.specs.is_empty() {
            self.specs = demo_specs(frame);
            self.skills = demo_skills(frame);
        }
    }
}
