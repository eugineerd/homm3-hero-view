use eframe::{egui, epi};

use crate::geometry::*;
use crate::hero::*;
use crate::utils::RawImage;

const H_GOLD: egui::Color32 = egui::Color32::from_rgb(248, 230, 194);

#[derive(Default)]
pub struct HeroViewer {
    static_assets: StaticAssets,
    heroes: Vec<Hero>,
    selected_hero_idx: usize,
}

#[derive(Default)]
struct StaticAssets {
    background: RawImage,
    pskill_attack: RawImage,
    pskill_defence: RawImage,
    pskill_mpower: RawImage,
    pskill_knowledge: RawImage,
    pskill_xp: RawImage,
    pskill_mana: RawImage,
}

impl StaticAssets {
    fn init(&mut self, frame: &mut epi::Frame<'_>) {
        self.background
            .load_bytes(include_bytes!("../resources/heroscr4.png"), frame);
        self.pskill_attack
            .load_bytes(include_bytes!("../resources/pskill_attack.png"), frame);
        self.pskill_defence
            .load_bytes(include_bytes!("../resources/pskill_defence.png"), frame);
        self.pskill_mpower
            .load_bytes(include_bytes!("../resources/pskill_mpower.png"), frame);
        self.pskill_knowledge
            .load_bytes(include_bytes!("../resources/pskill_knowledge.png"), frame);
        self.pskill_xp
            .load_bytes(include_bytes!("../resources/pskill_xp.png"), frame);
        self.pskill_mana
            .load_bytes(include_bytes!("../resources/pskill_mana.png"), frame);
    }
}

impl epi::App for HeroViewer {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.static_assets.init(frame);
        self.heroes = demo_heroes(frame);
        self.selected_hero_idx = 0;

        let mut visuals = egui::Visuals::default();
        visuals.override_text_color = Some(egui::Color32::WHITE);

        let mut style = egui::Style::default();
        style.spacing.button_padding = (0.0, 0.0).into();
        style.visuals = visuals;

        ctx.set_style(style);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        egui::Area::new("background").show(ctx, |ui| {
            ui.put(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(672., 586.)),
                self.static_assets.background.image(),
            );

            self.show_hero_switcher(ui);
            self.show_portrait_name_class(ui);
            self.show_primary_skills(ui);
            self.show_specialty(ui);
            self.show_skills(ui);
        });
    }

    fn name(&self) -> &str {
        "HoMM3 Hero Viewer"
    }
}

impl HeroViewer {
    fn hero(&self) -> &Hero {
        &self.heroes[self.selected_hero_idx]
    }

    fn show_hero_switcher(&mut self, ui: &mut egui::Ui) {
        for (idx, hero) in self.heroes.iter().enumerate() {
            if ui
                .put(
                    H_SWITCHER_PORTRAIT.translate(H_SWITCHER_PORTRAIT_OFFSET * idx as f32),
                    hero.character.portrait_small.image_button(),
                )
                .clicked()
            {
                self.selected_hero_idx = idx;
            }
        }
        let mut selected_frame = H_SWITCHER_PORTRAIT;
        selected_frame.min -= (1.0, 1.0).into();
        selected_frame.max += (1.0, 1.0).into();
        selected_frame =
            selected_frame.translate(H_SWITCHER_PORTRAIT_OFFSET * self.selected_hero_idx as f32);
        ui.painter()
            .rect_stroke(selected_frame, 0.0, egui::Stroke::new(1., H_GOLD));
    }

    fn show_portrait_name_class(&self, ui: &mut egui::Ui) {
        ui.put(H_PORTRAIT, self.hero().character.portrait.image());
        let hero_name_label = egui::Label::new(
            egui::RichText::new(&self.hero().character.name)
                .heading()
                .color(H_GOLD),
        );
        ui.put(H_NAME, hero_name_label);
        let hero_class_label = egui::Label::new(&format!(
            "{} {}-го уровня",
            &self.hero().character.class,
            self.hero().level
        ));
        ui.put(H_CLASS, hero_class_label);
    }

    fn show_specialty(&self, ui: &mut egui::Ui) {
        ui.put(SPEC_IMAGE, self.hero().specialty_image.image());
        let spec_top_label = egui::Label::new("Специальность");
        ui.allocate_ui_at_rect(SKILL_TEXT_TOP.translate(-SKILL_OFFSET_V), |ui| {
            ui.add(spec_top_label);
        });
        let spec_bottom_label = egui::Label::new(&self.hero().specialty_name);
        ui.allocate_ui_at_rect(SKILL_TEXT_BOTTOM.translate(-SKILL_OFFSET_V), |ui| {
            ui.add(spec_bottom_label);
        });
    }

    fn show_primary_skills(&self, ui: &mut egui::Ui) {
        #[rustfmt::skip]
        let pskill_images = [
            ("Атака", &self.static_assets.pskill_attack, self.hero().attack),
            ("Защита", &self.static_assets.pskill_defence, self.hero().defense),
            ("Магия", &self.static_assets.pskill_mpower, self.hero().mpower),
            ("Знания", &self.static_assets.pskill_knowledge, self.hero().knowledge),
        ];
        for (i, (name, image, value)) in pskill_images.iter().enumerate() {
            let offset = PSKILL_OFFSET * i as f32;
            ui.put(PSKILL_IMAGE.translate(offset), image.image());
            let name_label = egui::Label::new(egui::RichText::new(*name).color(H_GOLD));
            ui.put(PSKILL_NAME.translate(offset), name_label);
            let value_label = egui::Label::new(&format!("{}", value));
            ui.put(PSKILL_VALUE.translate(offset), value_label);
        }

        ui.put(SKILL_IMAGE, self.static_assets.pskill_xp.image());
        let xp_top_label = egui::Label::new("Опыт");
        ui.allocate_ui_at_rect(SKILL_TEXT_TOP, |ui| {
            ui.add(xp_top_label);
        });
        let xp_bottom_label = egui::Label::new(&format!("{}", self.hero().experience));
        ui.allocate_ui_at_rect(SKILL_TEXT_BOTTOM, |ui| {
            ui.add(xp_bottom_label);
        });

        ui.put(
            SKILL_IMAGE.translate(SKILL_OFFSET_H),
            self.static_assets.pskill_mana.image(),
        );
        let mana_top_label = egui::Label::new("Очки магии");
        ui.allocate_ui_at_rect(SKILL_TEXT_TOP.translate(SKILL_OFFSET_H), |ui| {
            ui.add(mana_top_label);
        });
        let mana_bottom_label = egui::Label::new(&format!(
            "{}/{}",
            self.hero().mana_current,
            self.hero().mana_max
        ));
        ui.allocate_ui_at_rect(SKILL_TEXT_BOTTOM.translate(SKILL_OFFSET_H), |ui| {
            ui.add(mana_bottom_label);
        });
    }

    fn show_skills(&self, ui: &mut egui::Ui) {}
}
