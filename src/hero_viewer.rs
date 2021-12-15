use eframe::{egui, epi};

use crate::backend::Backend;
use crate::edit_views::edit_mana_popup;
use crate::edit_views::edit_pskill_popup;
use crate::edit_views::edit_skill_popup;
use crate::edit_views::edit_spec_popup;
// use crate::edit_views::edit_spec_window;
use crate::edit_views::edit_xp_popup;
use crate::geometry::*;
use crate::hero::*;
use crate::skill::Skill;
use crate::spec::Spec;
use crate::utils::RawImage;

const H_GOLD: egui::Color32 = egui::Color32::from_rgb(248, 230, 194);

#[derive(Default)]
pub struct HeroViewer<B: Backend> {
    static_assets: StaticAssets,
    heroes: Vec<Hero>,
    hero_idx: usize,
    pixels_per_point: f32,
    backend: B,
}

#[derive(Default)]
pub struct StaticAssets {
    pub background: RawImage,
    pub pskills: [RawImage; 4],
    pub xp: RawImage,
    pub mana: RawImage,
    pub luck: [RawImage; 7],
    pub morale: [RawImage; 7],
    pub flag: RawImage,
}

impl StaticAssets {
    pub fn init(&mut self, frame: &mut epi::Frame<'_>) {
        self.background
            .load_bytes(include_bytes!("../resources/heroscr4.png"), frame);
        self.pskills[0].load_bytes(include_bytes!("../resources/pskill_attack.png"), frame);
        self.pskills[1].load_bytes(include_bytes!("../resources/pskill_defence.png"), frame);
        self.pskills[2].load_bytes(include_bytes!("../resources/pskill_mpower.png"), frame);
        self.pskills[3].load_bytes(include_bytes!("../resources/pskill_knowledge.png"), frame);
        self.xp
            .load_bytes(include_bytes!("../resources/pskill_xp.png"), frame);
        self.mana
            .load_bytes(include_bytes!("../resources/pskill_mana.png"), frame);
        self.flag
            .load_bytes(include_bytes!("../resources/crest.png"), frame);

        self.luck[0].load_bytes(include_bytes!("../resources/luck/00_00.png"), frame);
        self.luck[1].load_bytes(include_bytes!("../resources/luck/00_01.png"), frame);
        self.luck[2].load_bytes(include_bytes!("../resources/luck/00_02.png"), frame);
        self.luck[3].load_bytes(include_bytes!("../resources/luck/00_03.png"), frame);
        self.luck[4].load_bytes(include_bytes!("../resources/luck/00_04.png"), frame);
        self.luck[5].load_bytes(include_bytes!("../resources/luck/00_05.png"), frame);
        self.luck[6].load_bytes(include_bytes!("../resources/luck/00_06.png"), frame);

        self.morale[0].load_bytes(include_bytes!("../resources/morale/00_00.png"), frame);
        self.morale[1].load_bytes(include_bytes!("../resources/morale/00_01.png"), frame);
        self.morale[2].load_bytes(include_bytes!("../resources/morale/00_02.png"), frame);
        self.morale[3].load_bytes(include_bytes!("../resources/morale/00_03.png"), frame);
        self.morale[4].load_bytes(include_bytes!("../resources/morale/00_04.png"), frame);
        self.morale[5].load_bytes(include_bytes!("../resources/morale/00_05.png"), frame);
        self.morale[6].load_bytes(include_bytes!("../resources/morale/00_06.png"), frame);
    }
}

impl<B: Backend> epi::App for HeroViewer<B> {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.pixels_per_point = 1.0;

        self.static_assets.init(frame);
        self.heroes = demo_heroes(frame);
        self.hero_idx = 0;

        let mut visuals = egui::Visuals::default();
        visuals.override_text_color = Some(egui::Color32::WHITE);

        let mut style = egui::Style::default();
        style.spacing.button_padding = (0.0, 0.0).into();
        style.visuals = visuals;

        ctx.set_style(style);
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        self.process_global_hotkeys(ctx.input());

        self.backend.update(frame);

        ctx.set_pixels_per_point(self.pixels_per_point);
        frame.set_window_size(WINDOW_SIZE);

        egui::Area::new("background")
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                ui.put(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW_SIZE),
                    self.static_assets.background.image(),
                );

                self.show_hero_switcher(ui);
                self.show_flag(ui);
                self.show_portrait_name_class(ui);
                self.show_primary_skills(ui);
                self.show_xp(ui);
                self.show_mana(ui);
                self.show_specialty(ui);
                self.show_skills(ui);
                self.show_luck_morale(ui);
            });
    }

    fn name(&self) -> &str {
        "HoMM3 Hero Viewer"
    }
}

impl<B: Backend> HeroViewer<B> {
    fn hero(&self) -> &Hero {
        &self.heroes[self.hero_idx]
    }

    fn hero_mut(&mut self) -> &mut Hero {
        &mut self.heroes[self.hero_idx]
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
                self.hero_idx = idx;
            }
        }
        let selected_hero =
            H_SWITCHER_PORTRAIT.translate(H_SWITCHER_PORTRAIT_OFFSET * self.hero_idx as f32);
        selected_frame_around(ui, selected_hero);
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

    fn show_specialty(&mut self, ui: &mut egui::Ui) {
        let widget_response = ui.put(SPEC_IMAGE, self.hero().spec.image.image_button());
        let specs = self
            .backend
            .get_specs(&self.heroes[self.hero_idx].character.class);
        if let Some(new_value) = edit_spec_popup(
            ui,
            widget_response,
            &specs,
            &self.heroes[self.hero_idx].spec,
        ) {
            self.hero_mut().spec = new_value;
        }

        ui.allocate_ui_at_rect(SKILL_TEXT.translate(-SKILL_OFFSET_V), |ui| {
            let spec_top_label = egui::Label::new("Специальность");
            ui.add(spec_top_label);
            ui.add_space(4.);
            let spec_bottom_label = egui::Label::new(&self.hero().spec.name);
            ui.add(spec_bottom_label);
        });
    }

    fn show_primary_skills(&mut self, ui: &mut egui::Ui) {
        for (i, ((name, image), value)) in ["Атака", "Защита", "Магия", "Знания"]
            .iter()
            .zip(
                self.static_assets
                    .pskills
                    .iter()
                    .map(|i| i.image_button())
                    .collect::<Vec<_>>(),
            )
            .zip(&mut self.hero_mut().pskills)
            .enumerate()
        {
            let offset = PSKILL_OFFSET * i as f32;
            let image_rect = PSKILL_IMAGE.translate(offset);
            let name_rect = PSKILL_NAME.translate(offset);
            let value_rect = PSKILL_VALUE.translate(offset);

            let image_button_response = ui.put(image_rect, image);
            if let Some(new_value) = edit_pskill_popup(ui, image_button_response, *value) {
                *value = new_value;
            }

            let name_label = egui::Label::new(egui::RichText::new(*name).color(H_GOLD));
            ui.put(name_rect, name_label);
            let value_label = egui::Label::new(&value.to_string());
            ui.put(value_rect, value_label);
        }
    }

    fn show_skills(&mut self, ui: &mut egui::Ui) {
        for (i, skill) in self.heroes[self.hero_idx].skills.iter_mut().enumerate() {
            let offset = SKILL_OFFSET_V * (i % 4 + 1) as f32 + SKILL_OFFSET_H * (i / 4) as f32;
            let widget_response = if let Some(skill) = skill {
                let image = skill.image.image_button();
                ui.put(SKILL_IMAGE.translate(offset), image)
            } else {
                let button = egui::Button::new("").fill(egui::Color32::TRANSPARENT);
                ui.put(SKILL_IMAGE.translate(offset), button)
            };
            let skills = self.backend.get_skills();
            if let Some(new_value) = edit_skill_popup(ui, widget_response, skills, &skill, offset) {
                *skill = new_value;
            }

            if let Some(skill) = skill {
                ui.allocate_ui_at_rect(SKILL_TEXT.translate(offset), |ui| {
                    let spec_top_label = egui::Label::new(&skill.name);
                    ui.add(spec_top_label);
                    ui.add_space(4.);
                    let spec_bottom_label = egui::Label::new(&format!("{} ступени", skill.level));
                    ui.add(spec_bottom_label);
                });
            }
        }
    }

    fn show_luck_morale(&self, ui: &mut egui::Ui) {
        ui.put(
            LUCK_IMAGE,
            self.static_assets.luck[self.hero().luck as usize].image(),
        )
        .on_hover_text(
            [
                "Отряд проклят!",
                "Ужасная",
                "Плохая",
                "Нормальная",
                "Хорошая",
                "Отличная",
                "Великолепная",
            ][self.hero().luck as usize],
        );
        ui.put(
            MORALE_IMAGE,
            self.static_assets.morale[self.hero().morale as usize].image(),
        )
        .on_hover_text(
            [
                "Готовы предать",
                "Ужасная",
                "Плохая",
                "Нормальная",
                "Хорошая",
                "Отличная",
                "Ярость!",
            ][self.hero().morale as usize],
        );
    }

    fn show_units(&mut self, ui: &mut egui::Ui) {
        unimplemented!()
    }

    fn show_backpack(&mut self, ui: &mut egui::Ui) {
        unimplemented!()
    }

    fn show_flag(&mut self, ui: &mut egui::Ui) {
        if ui
            .put(FLAG_IMAGE, self.static_assets.flag.image_button())
            .clicked()
        {}
    }

    fn process_global_hotkeys(&mut self, input: &egui::InputState) {
        if input.key_pressed(egui::Key::X) && input.modifiers.ctrl {
            self.pixels_per_point = (self.pixels_per_point + 0.1).min(1.5);
        } else if input.key_pressed(egui::Key::Z) && input.modifiers.ctrl {
            self.pixels_per_point = (self.pixels_per_point - 0.1).max(1.0);
        }
    }

    fn show_xp(&mut self, ui: &mut egui::Ui) {
        let widget_response = ui.put(SKILL_IMAGE, self.static_assets.xp.image_button());
        if let Some(new_value) = edit_xp_popup(ui, widget_response, self.hero().experience) {
            self.hero_mut().experience = new_value;
        }
        ui.allocate_ui_at_rect(SKILL_TEXT, |ui| {
            let xp_top_label = egui::Label::new("Опыт");
            ui.add(xp_top_label);
            ui.add_space(4.);
            let xp_bottom_label = egui::Label::new(&self.hero().experience.to_string());
            ui.add(xp_bottom_label)
        });
    }

    fn show_mana(&mut self, ui: &mut egui::Ui) {
        let widget_response = ui.put(
            SKILL_IMAGE.translate(SKILL_OFFSET_H),
            self.static_assets.mana.image_button(),
        );
        if let Some((new_current, new_max)) = edit_mana_popup(
            ui,
            widget_response,
            self.hero().mana_current,
            self.hero().mana_max,
        ) {
            self.hero_mut().mana_current = new_current;
            self.hero_mut().mana_max = new_max;
        }
        ui.allocate_ui_at_rect(SKILL_TEXT.translate(SKILL_OFFSET_H), |ui| {
            let mana_top_label = egui::Label::new("Очки магии");
            ui.add(mana_top_label);
            ui.add_space(4.);
            let mana_bottom_label = egui::Label::new(&format!(
                "{}/{}",
                self.hero().mana_current,
                self.hero().mana_max
            ));
            ui.add(mana_bottom_label);
        });
    }
}

fn selected_frame_around(ui: &mut egui::Ui, mut rect: egui::Rect) {
    rect.min -= (1.0, 1.0).into();
    rect.max += (1.0, 1.0).into();
    ui.painter()
        .rect_stroke(rect, 0.0, egui::Stroke::new(1., H_GOLD));
}
