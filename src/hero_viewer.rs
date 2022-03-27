use eframe::{egui, epi};

use crate::backend::BackendStatus;
use crate::backend::DemoBackend;
use crate::geometry::*;
use crate::hero::*;
use crate::static_assets::StaticAssets;
use crate::utils::*;
use crate::widgets::*;

#[derive(Default)]
pub struct HeroViewer {
    static_assets: StaticAssets,
    hero_id: Option<usize>,
    hero: Option<Hero>,
    selected_hero_idx: usize,
    player: (usize, String),
    pixels_per_point: f32,
    search_query: String,
    backend: DemoBackend,
    backend_messages: Vec<String>,
}

impl epi::App for HeroViewer {
    fn setup(
        &mut self,
        ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.pixels_per_point = 1.0;

        self.static_assets.init(frame);

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
        if let Ok(msg) = self.backend.messages_receiver.try_recv() {
            self.backend_messages.push(msg);
        }
        if let Some(id) = self.hero_id {
            let new_hero = self.backend.get_hero(id);
            self.hero = Some(new_hero.unwrap().clone());
        }

        ctx.set_pixels_per_point(self.pixels_per_point);
        frame.set_window_size(WINDOW_SIZE);

        egui::Area::new("background")
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                ui.put(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, WINDOW_SIZE),
                    self.static_assets.background.image(),
                );

                self.show_settings(ui);
                if self.show_status(ui).is_none() {
                    return;
                }
                if ui
                    .put(
                        SKILL_BOX.translate(SKILL_OFFSET_V),
                        egui::Button::new("Clear all tables"),
                    )
                    .clicked()
                {
                    self.backend.clear_all_tables();
                }
                if ui
                    .put(
                        SKILL_BOX.translate(SKILL_OFFSET_V + SKILL_OFFSET_H),
                        egui::Button::new("Clear heroes"),
                    )
                    .clicked()
                {
                    self.backend.clear_heroes();
                }
                if ui
                    .put(
                        SKILL_BOX.translate(2. * SKILL_OFFSET_V + SKILL_OFFSET_H),
                        egui::Button::new("Init values"),
                    )
                    .clicked()
                {
                    self.backend.init_values();
                }

                self.show_hero_switcher(ui);
                self.show_portrait_name_class(ui);
                self.show_primary_skills(ui);
                self.show_xp(ui);
                self.show_mana(ui);
                self.show_specialty(ui, frame);
                self.show_skills(ui, frame);
                self.show_luck_morale(ui);
            });
    }

    fn name(&self) -> &str {
        "HoMM3 Hero Viewer"
    }
}

macro_rules! get_or_return {
    ($target:expr) => {
        match $target {
            Some(v) => v,
            None => return,
        }
    };
}

impl HeroViewer {
    fn show_hero_switcher(&mut self, ui: &mut egui::Ui) {
        for (idx, hero_button) in self
            .backend
            .get_player_heroes(self.player.0)
            .iter()
            .enumerate()
        {
            if ui
                .put(
                    H_SWITCHER_PORTRAIT.translate(H_SWITCHER_PORTRAIT_OFFSET * idx as f32),
                    hero_button.portrait.image_button(),
                )
                .clicked()
            {
                self.selected_hero_idx = idx;
                self.hero_id = Some(hero_button.id);
            }
        }
        if self.hero.is_some() {
            let selected_hero = H_SWITCHER_PORTRAIT
                .translate(H_SWITCHER_PORTRAIT_OFFSET * self.selected_hero_idx as f32);
            selected_frame_around(ui, selected_hero);
        }
    }

    fn show_portrait_name_class(&self, ui: &mut egui::Ui) {
        let hero = get_or_return!(&self.hero);

        ui.put(H_PORTRAIT, hero.character.portrait.image());
        let hero_name_label = egui::Label::new(
            egui::RichText::new(&hero.character.name)
                .heading()
                .color(H_GOLD),
        );
        ui.put(H_NAME, hero_name_label);
        let hero_class_label = egui::Label::new(&format!(
            "{} {}-го уровня",
            hero.character.class, hero.level
        ));
        ui.put(H_CLASS, hero_class_label);
    }

    fn show_specialty(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame) {
        let hero = get_or_return!(&mut self.hero);
        let mut set_new_value = false;

        let widget_response = ui.put(SPEC_IMAGE, hero.spec.image.image_button());

        let mut edit_value = None;

        show_selection_window(ui, widget_response, "Специалность", |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.search_query);
                if ui.button("➕").clicked() {
                    edit_value = Some(None);
                }
            });

            let rows = get_or_return!(self
                .backend
                .get_specs_row_count(hero.id, &self.search_query));
            let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, true]);
            scroll_area.show_rows(ui, SKILL_BOX.height(), rows, |ui, range| {
                let search_range = get_or_return!(self.backend.get_specs_range(
                    hero.id,
                    &self.search_query,
                    &range
                ));
                for s in search_range {
                    let selected = *s == hero.spec;
                    let (e, b) = show_selectable_block(ui, &s.image, &s.name, selected);
                    if e.clicked() {
                        edit_value = Some(Some(s.clone()));
                    } else if b.clicked() {
                        hero.spec = s.clone();
                        set_new_value = true;
                    }
                }
            });
        });

        ui.allocate_ui_at_rect(SKILL_TEXT.translate(-SKILL_OFFSET_V), |ui| {
            let spec_top_label = egui::Label::new("Специальность");
            ui.add(spec_top_label);
            ui.add_space(4.);
            let spec_bottom_label = egui::Label::new(&hero.spec.name);
            ui.add(spec_bottom_label);
        });

        show_spec_edit_window(ui, edit_value, &mut self.backend, frame);

        if set_new_value {
            self.backend.set_hero_spec(hero.id, &hero.spec.name);
        }
    }

    fn show_xp(&mut self, ui: &mut egui::Ui) {
        let hero = get_or_return!(&mut self.hero);

        let widget_response = ui.put(SKILL_IMAGE, self.static_assets.xp.image_button());
        if let Some(new_value) = show_xp_popup(ui, widget_response, hero.experience) {
            hero.experience = new_value;
            self.backend.set_hero_xp(hero.id, new_value);
        }
        ui.allocate_ui_at_rect(SKILL_TEXT, |ui| {
            let xp_top_label = egui::Label::new("Опыт");
            ui.add(xp_top_label);
            ui.add_space(4.);
            let xp_bottom_label = egui::Label::new(&hero.experience.to_string());
            ui.add(xp_bottom_label)
        });
    }

    fn show_mana(&mut self, ui: &mut egui::Ui) {
        let hero = get_or_return!(&mut self.hero);

        let widget_response = ui.put(
            SKILL_IMAGE.translate(SKILL_OFFSET_H),
            self.static_assets.mana.image_button(),
        );
        if let Some((new_current, new_max)) =
            show_mana_popup(ui, widget_response, hero.mana_current, hero.mana_max)
        {
            hero.mana_current = new_current;
            hero.mana_max = new_max;
            self.backend.set_hero_mana(hero.id, new_current, new_max);
        }
        ui.allocate_ui_at_rect(SKILL_TEXT.translate(SKILL_OFFSET_H), |ui| {
            let mana_top_label = egui::Label::new("Очки магии");
            ui.add(mana_top_label);
            ui.add_space(4.);
            let mana_bottom_label =
                egui::Label::new(&format!("{}/{}", hero.mana_current, hero.mana_max));
            ui.add(mana_bottom_label);
        });
    }

    fn show_primary_skills(&mut self, ui: &mut egui::Ui) {
        let hero = get_or_return!(&mut self.hero);

        for (i, ((name, image), value)) in ["Атака", "Защита", "Магия", "Знания"]
            .iter()
            .zip(
                self.static_assets
                    .pskills
                    .iter()
                    .map(|i| i.image_button())
                    .collect::<Vec<_>>(),
            )
            .zip(&mut hero.pskills)
            .enumerate()
        {
            let offset = PSKILL_OFFSET * i as f32;
            let image_rect = PSKILL_IMAGE.translate(offset);
            let name_rect = PSKILL_NAME.translate(offset);
            let value_rect = PSKILL_VALUE.translate(offset);

            let image_button_response = ui.put(image_rect, image);
            if let Some(new_value) = show_pskill_popup(ui, image_button_response, *value) {
                *value = new_value;
                self.backend.set_hero_pskill(hero.id, i, new_value);
            }

            let name_label = egui::Label::new(egui::RichText::new(*name).color(H_GOLD));
            ui.put(name_rect, name_label);
            let value_label = egui::Label::new(&value.to_string());
            ui.put(value_rect, value_label);
        }
    }

    fn show_skills(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame) {
        let hero = get_or_return!(&mut self.hero);

        let mut edit_skill = None;

        for (i, skill) in hero.skills.iter_mut().enumerate() {
            let offset = SKILL_OFFSET_V * (i % 4 + 1) as f32 + SKILL_OFFSET_H * (i / 4) as f32;

            let mut set_new_value = false;

            let widget_response = if let Some(skill) = skill {
                let image = skill.image.image_button();
                ui.put(SKILL_IMAGE.translate(offset), image)
            } else {
                let button = egui::Button::new("").fill(egui::Color32::TRANSPARENT);
                ui.put(SKILL_IMAGE.translate(offset), button)
            };

            show_selection_window(ui, widget_response, &format!("skill_{}", i), |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.search_query);
                    if ui.button("➕").clicked() {
                        edit_skill = Some(None);
                        return;
                    }
                    if ui.button("❌").clicked() {
                        *skill = None;
                        set_new_value = true;
                    }
                });

                let rows = get_or_return!(self
                    .backend
                    .get_skill_row_count(hero.id, &self.search_query));
                let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, true]);
                scroll_area.show_rows(ui, SKILL_BOX.height(), rows, |ui, range| {
                    let search_range = get_or_return!(self.backend.get_skill_range(
                        hero.id,
                        &self.search_query,
                        &range
                    ));
                    for s in search_range {
                        let is_selected =
                            skill.as_ref().map(|s2| s2.id == s.id).unwrap_or_default();
                        let (e, b) = show_selectable_block(
                            ui,
                            &s.image,
                            &format!("{}\n{} ступени", s.name, s.level),
                            is_selected,
                        );
                        if e.clicked() {
                            edit_skill = Some(Some(s.clone()));
                            return;
                        } else if b.clicked() {
                            *skill = Some(s.clone());
                            set_new_value = true;
                        }
                    }
                });

                if set_new_value {
                    self.backend
                        .set_hero_skill(hero.id, i, skill.as_ref().map(|s| s.id))
                }
            });

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
        show_skill_edit_window(ui, edit_skill, &mut self.backend, frame);
    }

    fn show_luck_morale(&mut self, ui: &mut egui::Ui) {
        let hero = get_or_return!(&mut self.hero);
        let luck_titles = [
            "Отряд проклят!",
            "Ужасная",
            "Плохая",
            "Нормальная",
            "Хорошая",
            "Отличная",
            "Великолепная",
        ];
        let morale_titles = [
            "Готовы предать",
            "Ужасная",
            "Плохая",
            "Нормальная",
            "Хорошая",
            "Отличная",
            "Ярость!",
        ];

        let luck_button_response = ui
            .put(
                LUCK_IMAGE,
                self.static_assets.luck[hero.luck as usize].image_button(),
            )
            .on_hover_text(luck_titles[hero.luck as usize]);
        show_selection_window(ui, luck_button_response, "Удача", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, title) in luck_titles.iter().enumerate() {
                    let selected = i as u8 == hero.luck;
                    if show_selectable_block_no_edit(
                        ui,
                        &self.static_assets.luck[i],
                        *title,
                        selected,
                    )
                    .clicked()
                    {
                        hero.luck = i as u8;
                        self.backend.set_hero_luck(hero.id, i as u8);
                    }
                }
            })
        });

        let morale_button_response = ui
            .put(
                MORALE_IMAGE,
                self.static_assets.morale[hero.morale as usize].image_button(),
            )
            .on_hover_text(morale_titles[hero.morale as usize]);
        show_selection_window(ui, morale_button_response, "Мораль", |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, title) in morale_titles.iter().enumerate() {
                    let selected = i as u8 == hero.morale;
                    if show_selectable_block_no_edit(
                        ui,
                        &self.static_assets.morale[i],
                        *title,
                        selected,
                    )
                    .clicked()
                    {
                        hero.morale = i as u8;
                        self.backend.set_hero_morale(hero.id, i as u8);
                    }
                }
            })
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        use egui::*;
        use BackendStatus::*;
        let button_response = ui.put(FLAG_IMAGE, self.static_assets.flag.image_button());
        show_selection_window(ui, button_response, "Настройки", |ui| {
            let status = self.backend.get_status();
            Grid::new("grid").num_columns(2).show(ui, |ui| {
                ui.label("Подключиться к БД");
                let connect_button = ui.add_enabled(status == NotConnected, Button::new("Connect"));
                if connect_button.clicked() {
                    self.backend.connect_to_db();
                }
                ui.end_row();

                ui.label("Создать БД");
                let create_db_button =
                    ui.add_enabled(status == NotConnected, Button::new("Create DB"));
                if create_db_button.clicked() {
                    self.backend.create_db();
                }
                ui.end_row();

                ui.label("Удалить БД");
                let drop_db_button = ui.add_enabled(
                    status == Idle || status == NotConnected,
                    Button::new("Drop DB"),
                );
                if drop_db_button.clicked() {
                    self.backend.drop_db();
                }
                ui.end_row();

                ui.label("Игрок");
                egui::ComboBox::from_id_source(ui.id())
                    .selected_text(&self.player.1)
                    .show_ui(ui, |ui| {
                        for player in self.backend.get_players() {
                            ui.selectable_value(&mut self.player, player.clone(), &player.1);
                        }
                    });
                ui.end_row();

                ui.label("Размер интерфейса");
                ui.add(Slider::new(&mut self.pixels_per_point, 1.0..=1.5));
                ui.end_row()
            });
        })
    }

    fn process_global_hotkeys(&mut self, input: &egui::InputState) {
        if input.key_pressed(egui::Key::X) && input.modifiers.ctrl {
            self.pixels_per_point = (self.pixels_per_point + 0.1).min(1.5);
        } else if input.key_pressed(egui::Key::Z) && input.modifiers.ctrl {
            self.pixels_per_point = (self.pixels_per_point - 0.1).max(1.0);
        }
    }

    fn show_status(&self, ui: &mut egui::Ui) -> Option<()> {
        let status = self.backend.get_status();
        if status == BackendStatus::NotConnected {
            ui.put(
                INFO_BOX,
                egui::Label::new(format!(
                    "Not connected. {}",
                    self.backend_messages.last().unwrap_or(&"".to_string())
                )),
            );
            None
        } else if status == BackendStatus::Connecting {
            ui.put(INFO_BOX, egui::Label::new("Connecting..."));
            None
        } else if let Some(msg) = self.backend_messages.last() {
            ui.put(INFO_BOX, egui::Label::new(msg));
            Some(())
        } else {
            None
        }
    }
}
