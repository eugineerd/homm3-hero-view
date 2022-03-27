use eframe::{egui, epi};
use egui::*;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(sqlx::FromRow)]
struct RawHero {
    id: i32,
    level: i32,
    experience: i32,
    offence: i32,
    defence: i32,
    mana: i32,
    knowledge: i32,
    name: String,
    side: String,
    class: String,
    // slot: i32,
    // amount: i32,
    // unit_name: String,
    // unit_level: i32,
    // unit_offence: i32,
    // unit_defence: i32,
    // unit_shots: i32,
    // unit_damage: String,
    // unit_vital: i32,
    // unit_speed: i32,
}

// #[derive(Clone)]
// struct Unit {
//     amount: i32,
//     name: String,
//     level: i32,
//     offence: i32,
//     defence: i32,
//     shots: i32,
//     damage: String,
//     vital: i32,
//     speed: i32,
// }

#[derive(Clone)]
struct Hero {
    id: i32,
    level: i32,
    exp: i32,
    offence: i32,
    defense: i32,
    mana: i32,
    knowledge: i32,
    name: String,
    side: String,
    class: String,
    // units: [Option<Unit>; 7],
}

#[derive(Default)]
struct SharedState {
    heroes: Vec<Hero>,
    players: Vec<(i32, String)>,
    characters: Vec<(i32, String)>,
    log: Vec<String>,
    selected_player: Option<i32>,
    repaint: Option<Arc<dyn epi::RepaintSignal>>,
    db: Option<sqlx::Pool<sqlx::Postgres>>,
}

struct HeroViewer {
    tokio_rt: tokio::runtime::Runtime,
    shared_state: Arc<Mutex<SharedState>>,
}

impl HeroViewer {
    pub fn new() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        println!("Connecting to db...");
        let db = tokio_rt
            .block_on(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect("postgres://homm3:homm3@localhost/homm3_heroes"),
            )
            .ok();
        let mut s: SharedState = Default::default();
        s.db = db;
        Self {
            tokio_rt,
            shared_state: Arc::new(Mutex::new(s)),
        }
    }

    fn create_db(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let connect_result = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect("postgres://homm3:homm3@localhost/homm3")
                .await;
            let homm3_db = match connect_result {
                Err(e) => {
                    let mut h = state_handle.lock();
                    h.log.push(e.to_string());
                    h.repaint.as_ref().unwrap().request_repaint();
                    return;
                }
                Ok(db) => db,
            };

            let query = sqlx::query("CREATE DATABASE homm3_heroes;")
                .execute(&homm3_db)
                .await;
            if let Err(e) = query {
                let mut h = state_handle.lock();
                h.log.push(e.to_string());
                h.repaint.as_ref().unwrap().request_repaint();
            } else {
                let mut h = state_handle.lock();
                h.log.push("DB created".into());
                h.repaint.as_ref().unwrap().request_repaint();
            }

            let db = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect("postgres://homm3:homm3@localhost/homm3_heroes")
                .await;
            let db = match db {
                Err(e) => {
                    let mut h = state_handle.lock();
                    h.log.push(e.to_string());
                    h.repaint.as_ref().unwrap().request_repaint();
                    return;
                }
                Ok(v) => v,
            };
            let commands = include_str!("init2.sql").split("--<");
            for command in commands {
                let query = sqlx::query(command).execute(&db).await;
                if let Err(e) = query {
                    let mut h = state_handle.lock();
                    h.log.push(e.to_string());
                    h.repaint.as_ref().unwrap().request_repaint();
                }
            }
            let mut h = state_handle.lock();
            h.log.push("DB initialized".into());
            h.repaint.as_ref().unwrap().request_repaint();
            h.db = Some(db);
        });
    }

    fn get_players(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(get_players(state_handle.clone()));
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query_as::<_, (i32, String)>("SELECT get_characters()")
                .fetch_all(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(v) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.characters.extend(v.into_iter());
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
        });
    }

    fn get_heroes(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(get_heroes(state_handle));
    }

    fn delete_player(&self, id: i32) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query("SELECT delete_player($1)")
                .bind(id)
                .execute(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(_) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Removed player {}", id));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            state_handle.lock().heroes.clear();
            get_players(state_handle.clone()).await;
            get_heroes(state_handle).await;
        });
    }

    fn create_player(&self, name: String) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query_as::<_, (i32,)>("SELECT * FROM create_player($1)")
                .bind(name)
                .fetch_one(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(v) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.selected_player = Some(v.0);
                    state_lock.log.push(format!("Added player {}", v.0));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_players(state_handle.clone()).await;
            get_heroes(state_handle).await;
        });
    }

    fn set_hero_values(&self, hero: Hero) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query("SELECT modify_hero($1, $2, $3, $4, $5, $6)")
                .bind(hero.id)
                .bind(hero.exp)
                .bind(hero.offence)
                .bind(hero.defense)
                .bind(hero.mana)
                .bind(hero.knowledge)
                .execute(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(_) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Modified hero {}", hero.id));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_heroes(state_handle).await;
        });
    }

    fn create_hero(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            let selected_player = state_handle.lock().selected_player.unwrap();
            match sqlx::query_as::<_, (i32,)>("SELECT create_hero($1, 1)")
                .bind(selected_player)
                .fetch_one(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(v) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Added hero {}", v.0));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_heroes(state_handle).await;
        });
    }

    fn delete_hero(&self, id: i32) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query("SELECT delete_hero($1)")
                .bind(id)
                .execute(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(_) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Removed hero {}", id));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_heroes(state_handle).await;
        });
    }

    fn clear_players(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query("SELECT clear_players()")
                .execute(&db_handle)
                .await
            {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(_) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Cleared all players"));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_heroes(state_handle.clone()).await;
            get_players(state_handle).await;
        });
    }

    fn clear_hero(&self) {
        let state_handle = self.shared_state.clone();
        self.tokio_rt.spawn(async move {
            let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
            match sqlx::query("SELECT clear_hero()").execute(&db_handle).await {
                Err(e) => state_handle.lock().log.push(e.to_string()),
                Ok(_) => {
                    let mut state_lock = state_handle.lock();
                    state_lock.log.push(format!("Cleared all heroes"));
                    state_lock.repaint.as_ref().unwrap().request_repaint();
                }
            }
            get_heroes(state_handle).await;
        });
    }
}

impl epi::App for HeroViewer {
    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        frame: &mut epi::Frame<'_>,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.shared_state
            .lock()
            .repaint
            .replace(frame.repaint_signal());
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        let mut state_lock = self.shared_state.lock();
        let mut will_drop_db = false;
        TopBottomPanel::top("Top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Connection", |ui| {
                    if ui.button("Create DB").clicked() {
                        self.create_db();
                        ui.close_menu()
                    }
                    ui.add_enabled_ui(state_lock.db.is_some(), |ui| {
                        if ui.button("Drop DB").clicked() {
                            will_drop_db = true;
                            ui.close_menu();
                        }
                        if ui.button("Get players").clicked() {
                            self.get_players();
                            ui.close_menu();
                        }
                    });
                });
                ui.add_enabled_ui(!state_lock.players.is_empty(), |ui| {
                    ui.menu_button("Players", |ui| {
                        ui.set_width(ui.available_width());
                        let mut selected_player = state_lock.selected_player;
                        for (id, name) in state_lock.players.iter() {
                            ui.horizontal(|ui| {
                                let size = vec2(
                                    ui.available_width() - 20. + ui.style().spacing.item_spacing.x,
                                    20.,
                                );
                                if ui
                                    .add_sized(size, Button::new(format!("{} ({})", name, id)))
                                    .clicked()
                                {
                                    selected_player = Some(*id);
                                    self.get_heroes();
                                    ui.close_menu();
                                }
                                if ui.add_sized(vec2(20., 20.), Button::new("❌")).clicked() {
                                    selected_player = None;
                                    self.delete_player(*id);
                                    ui.close_menu();
                                };
                            });
                        }
                        ui.separator();
                        ui.horizontal(|ui| {
                            let textbox_id = ui.make_persistent_id("hero_add_textbox");
                            let mut textbox_string = ui
                                .memory()
                                .data
                                .get_temp_mut_or_default::<String>(textbox_id)
                                .clone();
                            ui.add(
                                TextEdit::singleline(&mut textbox_string)
                                    .desired_width(ui.available_width() - 20. - 6.),
                            );
                            if ui
                                .add_sized(vec2(ui.available_width(), 20.), Button::new("➕"))
                                .clicked()
                            {
                                self.create_player(textbox_string.clone());
                                textbox_string.clear();
                                ui.close_menu();
                            };
                            ui.memory().data.insert_temp(textbox_id, textbox_string);
                        });
                        state_lock.selected_player = selected_player;
                    });
                });
                ui.add_enabled_ui(state_lock.db.is_some(), |ui| {
                    ui.menu_button("Таблицы", |ui| {
                        if ui.button("Очистить игроков").clicked() {
                            self.clear_players();
                            ui.close_menu();
                        }
                        if ui.button("Очистить героев").clicked() {
                            self.clear_hero();
                            ui.close_menu();
                        }
                    })
                });
            })
        });
        TopBottomPanel::bottom("Log").show(ctx, |ui| {
            ui.set_height(50.);
            ScrollArea::vertical()
                .stick_to_bottom()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (i, line) in state_lock.log.iter().enumerate() {
                        ui.label(format!("[{}] {}", i, line));
                    }
                });
        });
        CentralPanel::default().show(ctx, |ui| {
            Grid::new("Hero Grid")
                .min_col_width((ui.available_width() - ui.style().spacing.item_spacing.x) / 4.)
                .min_row_height(ui.available_height() / 2.)
                .num_columns(4)
                .show(ui, |ui| {
                    for (i, hero) in state_lock.heroes.iter_mut().enumerate() {
                        Frame::default()
                            .shadow(epaint::Shadow::small_dark())
                            .show(ui, |ui| {
                                ui.set_width(
                                    ui.available_width() - ui.style().spacing.item_spacing.x,
                                );
                                ui.horizontal_top(|ui| self.show_hero(ui, hero))
                            });
                        if (i + 1) % 4 == 0 {
                            ui.end_row()
                        }
                    }
                    if state_lock.selected_player.is_some() && state_lock.heroes.len() < 8 {
                        ui.centered_and_justified(|ui| {
                            if ui.button("+").clicked() {
                                self.create_hero();
                            }
                        });
                    }
                })
        });

        if will_drop_db {
            drop_db(self.tokio_rt.handle().clone(), self.shared_state.clone());
        }
    }

    fn name(&self) -> &str {
        "HoMM3 hero viewer"
    }
}

impl HeroViewer {
    fn show_hero(&self, ui: &mut Ui, hero: &mut Hero) {
        Grid::new(hero.id)
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("ID");
                ui.label(hero.id.to_string());
                ui.end_row();

                ui.label("Имя");
                ui.label(&hero.name);
                ui.end_row();

                ui.label("Класс");
                ui.label(&hero.class);
                ui.end_row();

                ui.label("Сторона");
                ui.label(&hero.side);
                ui.end_row();

                ui.label("Уровень");
                ui.label(hero.level.to_string());
                ui.end_row();

                ui.label("Опыт");
                let mut s = hero.exp.to_string();
                let t = TextEdit::singleline(&mut s).desired_width(50.);
                if ui.add(t).changed() {
                    if let Ok(v) = s.parse::<u16>() {
                        hero.exp = v as i32;
                    }
                };
                ui.end_row();

                ui.label("Атака");
                let mut s = hero.offence.to_string();
                let t = TextEdit::singleline(&mut s).desired_width(50.);
                if ui.add(t).changed() {
                    if let Ok(v) = s.parse::<u16>() {
                        hero.offence = v as i32;
                    }
                };
                ui.end_row();

                ui.label("Защита");
                let mut s = hero.defense.to_string();
                let t = TextEdit::singleline(&mut s).desired_width(50.);
                if ui.add(t).changed() {
                    if let Ok(v) = s.parse::<u16>() {
                        hero.defense = v as i32;
                    }
                };
                ui.end_row();

                ui.label("Мана");
                let mut s = hero.mana.to_string();
                let t = TextEdit::singleline(&mut s).desired_width(50.);
                if ui.add(t).changed() {
                    if let Ok(v) = s.parse::<u16>() {
                        hero.mana = v as i32;
                    }
                };
                ui.end_row();

                ui.label("Знания");
                let mut s = hero.knowledge.to_string();
                let t = TextEdit::singleline(&mut s).desired_width(50.);
                if ui.add(t).changed() {
                    if let Ok(v) = s.parse::<u16>() {
                        hero.knowledge = v as i32;
                    }
                };
                ui.end_row();

                if ui.button("Save changes").clicked() {
                    self.set_hero_values(hero.clone());
                }
                if ui.button("Delete hero").clicked() {
                    self.delete_hero(hero.id);
                }
                ui.end_row();
            })
            .inner
    }
}

fn drop_db(rt: tokio::runtime::Handle, state_handle: Arc<Mutex<SharedState>>) {
    rt.spawn(async move {
        let h = state_handle.lock().db.take().unwrap();
        h.close().await;

        let connect_result = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect("postgres://homm3:homm3@localhost/homm3")
            .await;
        let homm3_db = match connect_result {
            Err(e) => {
                let mut h = state_handle.lock();
                h.log.push(e.to_string());
                h.repaint.as_ref().unwrap().request_repaint();
                return;
            }
            Ok(db) => db,
        };

        let query = sqlx::query("DROP DATABASE homm3_heroes;")
            .execute(&homm3_db)
            .await;
        if let Err(e) = query {
            let mut h = state_handle.lock();
            h.log.push(e.to_string());
            h.repaint.as_ref().unwrap().request_repaint();
        } else {
            let mut h = state_handle.lock();
            h.log.push("DB dropped".into());
            h.repaint.as_ref().unwrap().request_repaint();
        }

        let mut h = state_handle.lock();
        h.players.clear();
        h.selected_player = None;
        h.heroes.clear();
    });
}

async fn get_players(state_handle: Arc<Mutex<SharedState>>) {
    let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
    match sqlx::query_as("SELECT * FROM get_players()")
        .fetch_all(&db_handle)
        .await
    {
        Err(e) => state_handle.lock().log.push(e.to_string()),
        Ok(v) => {
            let mut state_lock = state_handle.lock();
            state_lock.log.push("Got all players".into());
            state_lock.repaint.as_ref().unwrap().request_repaint();
            state_lock.players = v;
        }
    }
}

async fn get_heroes(state_handle: Arc<Mutex<SharedState>>) {
    let db_handle = state_handle.lock().db.as_ref().unwrap().clone();
    let selected_player = state_handle.lock().selected_player.unwrap();
    match sqlx::query_as::<_, RawHero>("SELECT * FROM get_heroes($1)")
        .bind(selected_player)
        .fetch_all(&db_handle)
        .await
    {
        Err(e) => state_handle.lock().log.push(e.to_string()),
        Ok(v) => {
            let mut heroes: Vec<Hero> = Vec::new();
            for h in v {
                // let unit = Unit {
                //     amount: h.amount,
                //     name: h.unit_name,
                //     level: h.unit_level,
                //     offence: h.unit_offence,
                //     defence: h.unit_defence,
                //     shots: h.unit_shots,
                //     damage: h.unit_damage,
                //     vital: h.unit_vital,
                //     speed: h.unit_speed,
                // };

                // if let Some(hero) = heroes.last_mut() {
                //     if hero.id == h.id {
                //         hero.units.get_mut(h.slot as usize).replace(&mut Some(unit));
                //         continue;
                //     }
                // }
                // let mut units: [Option<Unit>; 7] = Default::default();
                // units.get_mut(h.slot as usize).replace(&mut Some(unit));
                heroes.push(Hero {
                    id: h.id,
                    level: h.level,
                    exp: h.experience,
                    offence: h.offence,
                    defense: h.defence,
                    mana: h.mana,
                    knowledge: h.knowledge,
                    name: h.name,
                    side: h.side,
                    class: h.class,
                    // units,
                });
            }

            let mut state_lock = state_handle.lock();
            let s = state_lock.selected_player.unwrap_or_default();
            state_lock.log.push(format!("Got heroes of player {}", s));
            state_lock.repaint.as_ref().unwrap().request_repaint();
            state_lock.heroes = heroes;
        }
    }
}

fn main() {
    let app = HeroViewer::new();
    eframe::run_native(Box::new(app), Default::default());
}
