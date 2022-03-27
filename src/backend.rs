use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

use crate::character::Character;
use crate::hero::{Hero, HeroSelectButton};
use crate::skill::Skill;
use crate::spec::Spec;
use crate::utils::RawImage;

pub struct DemoBackend {
    selected_player: Arc<Mutex<usize>>,
    pending_players: Arc<Mutex<Vec<(usize, String)>>>,
    cached_players: Vec<(usize, String)>,

    pending_hsb: Arc<Mutex<Vec<HeroSelectButton>>>,
    cached_hsb: Vec<HeroSelectButton>,
    reset_hsb_cache: bool,

    pending_hero: Arc<Mutex<Option<Hero>>>,
    cached_heroes: HashMap<usize, Hero>,
    reset_heroes_cache: bool,

    pending_specs: Arc<Mutex<Vec<Spec>>>,
    cached_specs: Vec<Spec>,
    specs_query: Arc<Mutex<(String, String)>>,
    reset_specs_cache: bool,

    pending_skills: Arc<Mutex<Vec<Skill>>>,
    cached_skills: Vec<Skill>,
    skills_query: Arc<Mutex<String>>,
    reset_skills_cache: bool,

    tokio_rt: tokio::runtime::Runtime,
    db_pool: Arc<Mutex<Option<sqlx::pool::Pool<sqlx::postgres::Postgres>>>>,
    repaint_signal: Option<Arc<dyn eframe::epi::RepaintSignal>>,
    status: Arc<Mutex<BackendStatus>>,
    pub messages_receiver: Receiver<String>,
    messages_sender: Sender<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BackendStatus {
    NotConnected,
    Connecting,
    Idle,
    QueryInProgress,
}

impl Default for BackendStatus {
    fn default() -> Self {
        Self::NotConnected
    }
}

impl Default for DemoBackend {
    fn default() -> Self {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (messages_sender, messages_receiver) = channel();

        Self {
            cached_specs: Default::default(),
            cached_skills: Default::default(),
            tokio_rt,
            repaint_signal: Default::default(),
            status: Default::default(),
            messages_receiver,
            messages_sender,
            db_pool: Default::default(),
            cached_heroes: Default::default(),
            specs_query: Default::default(),
            skills_query: Default::default(),
            selected_player: Default::default(),
            cached_hsb: Default::default(),
            pending_hsb: Default::default(),
            pending_hero: Default::default(),
            pending_specs: Default::default(),
            pending_skills: Default::default(),
            pending_players: Default::default(),
            cached_players: Default::default(),
            reset_hsb_cache: Default::default(),
            reset_heroes_cache: Default::default(),
            reset_specs_cache: Default::default(),
            reset_skills_cache: Default::default(),
        }
    }
}

impl DemoBackend {
    pub fn update(&mut self, frame: &mut eframe::epi::Frame) {
        self.repaint_signal
            .get_or_insert_with(|| frame.repaint_signal());
        if self.db_pool.lock().is_none() {
            return;
        }

        let mut buttons_lock = self.pending_hsb.lock();
        if !buttons_lock.is_empty() {
            self.cached_hsb = buttons_lock
                .drain(..)
                .map(|mut hsb| {
                    hsb.portrait.load_stored_bytes(frame);
                    hsb
                })
                .collect();
        }

        if let Some(mut hero) = self.pending_hero.lock().take() {
            // hero.character.portrait.load_stored_bytes(frame);
            self.cached_heroes.insert(hero.id, hero);
        }

        let mut players_lock = self.pending_players.lock();
        if !players_lock.is_empty() {
            self.cached_players = players_lock.drain(..).collect();
        }
    }

    pub fn get_status(&self) -> BackendStatus {
        self.status.lock().clone()
    }

    pub fn connect_to_db(&mut self) {
        let db_handle = self.db_pool.clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock() = BackendStatus::Connecting;
        repaint.request_repaint();

        self.tokio_rt.spawn(async move {
            let connect_result = sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect("postgres://homm3:homm3@localhost/homm3_heroes")
                .await;

            match connect_result {
                Err(e) => {
                    *status.lock() = BackendStatus::NotConnected;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                }
                Ok(db) => {
                    db_handle.lock().replace(db);

                    *status.lock() = BackendStatus::Idle;
                    msgs.send("Connected".into()).unwrap();
                    repaint.request_repaint();
                }
            };
        });
    }

    pub fn create_db(&mut self) {
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock() = BackendStatus::Connecting;
        repaint.request_repaint();

        self.tokio_rt.spawn(async move {
            let connect_result = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect("postgres://homm3:homm3@localhost/homm3")
                .await;
            let homm3_db = match connect_result {
                Err(e) => {
                    *status.lock() = BackendStatus::NotConnected;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                    return;
                }
                Ok(db) => db,
            };
            let query = sqlx::query("CREATE DATABASE homm3_heroes;")
                .execute(&homm3_db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::NotConnected;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::NotConnected;
                msgs.send("DB Created".into()).unwrap();
                repaint.request_repaint();
            }
        });
    }

    pub fn drop_db(&mut self) {
        let db_handle = self.db_pool.lock().take().unwrap();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock() = BackendStatus::Connecting;
        msgs.send("Dropping DB...".to_string()).unwrap();
        repaint.request_repaint();

        self.tokio_rt.spawn(async move {
            db_handle.close().await;
            let connect_result = sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect("postgres://homm3:homm3@localhost/homm3")
                .await;
            let homm3_db = match connect_result {
                Err(e) => {
                    *status.lock() = BackendStatus::NotConnected;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                    return;
                }
                Ok(db) => db,
            };

            let query = sqlx::query("DROP DATABASE homm3_heroes;")
                .execute(&homm3_db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::NotConnected;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::NotConnected;
                msgs.send("DB Dropped successfully".into()).unwrap();
                repaint.request_repaint();
            }
        });
    }
}

impl DemoBackend {
    pub fn get_players(&mut self) -> &[(usize, String)] {
        if !self.cached_players.is_empty() {
            return &self.cached_players;
        }

        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock != BackendStatus::Idle {
            return &self.cached_players;
        }

        let db_handle = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let players = self.pending_players.clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query_as::<_, (i32, String)>("SELECT * FROM get_players()")
                .fetch_all(&db_handle)
                .await;
            match query {
                Err(e) => {
                    *status.lock() = BackendStatus::Idle;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                }
                Ok(v) => {
                    *players.lock() = v.into_iter().map(|(a, b)| (a as usize, b)).collect();

                    *status.lock() = BackendStatus::Idle;
                    repaint.request_repaint();
                }
            };
        });

        &self.cached_players
    }

    pub fn get_player_heroes(&mut self, player_id: usize) -> &[HeroSelectButton] {
        let selected_player_lock = self.selected_player.lock();
        if *selected_player_lock == player_id {
            if !self.reset_hsb_cache {
                return &self.cached_hsb;
            }
        }

        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock != BackendStatus::Idle {
            return &self.cached_hsb;
        }
        self.reset_hsb_cache = false;

        std::mem::drop(selected_player_lock);

        *self.selected_player.lock() = player_id;

        let db_handle = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let pending = self.pending_hsb.clone();

        *status_lock = BackendStatus::QueryInProgress;
        msgs.send(format!(
            "Getting heroes for player with id {}...",
            player_id
        ))
        .unwrap();
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query_as::<_, (i32, Vec<u8>)>("SELECT * FROM get_player_heroes($1)")
                .bind(player_id as i32)
                .fetch_all(&db_handle)
                .await;
            match query {
                Err(e) => {
                    *status.lock() = BackendStatus::Idle;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                }
                Ok(v) => {
                    let new_hsb = v
                        .into_iter()
                        .map(|(hero_id, portrait)| HeroSelectButton {
                            id: hero_id as usize,
                            portrait: RawImage::take_bytes(portrait),
                        })
                        .collect();
                    *pending.lock() = new_hsb;

                    *status.lock() = BackendStatus::Idle;
                    msgs.send(format!("Got all heroes for player with id {}", player_id))
                        .unwrap();
                    repaint.request_repaint();
                }
            };
        });

        &self.cached_hsb
    }

    pub fn get_hero(&mut self, hero_id: usize) -> Option<&Hero> {
        if let Some(hero) = self.cached_heroes.get(&hero_id) {
            if !self.reset_heroes_cache {
                return Some(&hero);
            }
        }

        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock != BackendStatus::Idle {
            return self.cached_heroes.get(&hero_id);
        }

        self.reset_heroes_cache = false;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let pending = self.pending_hero.clone();

        *status_lock = BackendStatus::QueryInProgress;
        msgs.send(format!("Getting hero with id {}...", hero_id))
            .unwrap();
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query_as::<
                _,
                (
                    String,
                    Vec<u8>,
                    String,
                    i32,
                    i32,
                    i32,
                    i32,
                    i32,
                    Vec<i32>,
                    String,
                    Vec<u8>,
                    i32,
                ),
            >("SELECT * FROM get_hero($1)")
            .bind(hero_id as i32)
            .fetch_one(&db)
            .await;
            match query {
                Err(e) => {
                    *status.lock() = BackendStatus::Idle;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                }
                Ok(v) => {
                    let new_hero = Hero {
                        id: hero_id,
                        character: Character {
                            name: v.0,
                            portrait: RawImage::take_bytes(v.1),
                            class: v.2.clone(),
                        },
                        mana_current: v.3 as u16,
                        mana_max: v.4 as u16,
                        experience: v.5 as u16,
                        luck: v.6 as u8,
                        morale: v.7 as u8,
                        pskills: [v.8[0] as u8, v.8[1] as u8, v.8[2] as u8, v.8[3] as u8],
                        spec: Spec {
                            class: v.2,
                            name: v.9,
                            image: RawImage::take_bytes(v.10),
                        },
                        level: v.11 as u8,
                        skills: Default::default(),
                    };
                    *pending.lock() = Some(new_hero);

                    *status.lock() = BackendStatus::Idle;
                    msgs.send(format!("Got hero with id {} successfully", hero_id))
                        .unwrap();
                    repaint.request_repaint();
                }
            };
        });
        self.cached_heroes.get(&hero_id)
    }

    pub fn get_classes(&mut self) -> Option<Vec<String>> {
        // Some(vec!["Путешественник".to_string(), "Алхимик".to_string()])
        None
    }

    pub fn set_hero_pskill(&mut self, hero_id: usize, pskill: usize, value: u8) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT set_hero_pskill($1, $2, $3)")
                .bind(hero_id as i32)
                .bind(pskill as i32 + 1)
                .bind(value as i32)
                .execute(&db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("Primary skill successfully set".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn init_values(&mut self) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;
        self.reset_hsb_cache = true;
        self.reset_specs_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT initial_data()").execute(&db).await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("init values".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn clear_all_tables(&mut self) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;
        self.reset_hsb_cache = true;
        self.reset_specs_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT clear_all_tables()").execute(&db).await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("All tables cleared".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn clear_heroes(&mut self) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;
        self.reset_hsb_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT clear_heroes()").execute(&db).await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("All heroes cleared".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn set_hero_xp(&mut self, hero_id: usize, value: u16) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT set_hero_xp($1, $2)")
                .bind(hero_id as i32)
                .bind(value as i32)
                .execute(&db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("XP successfully set".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn set_hero_mana(&mut self, hero_id: usize, current_value: u16, max_value: u16) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        msgs.send(format!("Setting mana to: {}/{}", current_value, max_value))
            .unwrap();
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT set_hero_mana($1, $2, $3)")
                .bind(hero_id as i32)
                .bind(current_value as i32)
                .bind(max_value as i32)
                .execute(&db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("Mana successfully set".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn set_hero_morale(&mut self, hero_id: usize, value: u8) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT set_hero_morale($1, $2)")
                .bind(hero_id as i32)
                .bind(value as i32)
                .execute(&db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("Morale successfully set".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }

    pub fn set_hero_luck(&mut self, hero_id: usize, value: u8) {
        let status = self.status.clone();
        let mut status_lock = status.lock();
        if *status_lock == BackendStatus::QueryInProgress {
            return;
        }

        self.reset_heroes_cache = true;

        let db = self.db_pool.lock().as_ref().unwrap().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();

        *status_lock = BackendStatus::QueryInProgress;
        repaint.request_repaint();
        std::mem::drop(status_lock);

        self.tokio_rt.spawn(async move {
            let query = sqlx::query("SELECT set_hero_luck($1, $2)")
                .bind(hero_id as i32)
                .bind(value as i32)
                .execute(&db)
                .await;
            if let Err(e) = query {
                *status.lock() = BackendStatus::Idle;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock() = BackendStatus::Idle;
                msgs.send("Luck successfully set".into()).unwrap();
                repaint.request_repaint();
            };
        });
    }
}

impl<'a> DemoBackend {
    pub fn create_or_modify_spec(&mut self, name: &str, class: &str, image: &[u8]) {}

    pub fn get_specs_row_count(&mut self, hero_id: usize, query: &str) -> Option<usize> {
        // Some(
        //     self.hero_specs[&hero_id]
        //         .iter()
        //         .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        //         .count(),
        // )
        None
    }

    pub fn get_specs_range(
        &'a mut self,
        hero_id: usize,
        query: &'a str,
        range: &'a Range<usize>,
        // ) -> Option<impl Iterator<Item = &'a Spec>> {
    ) -> Option<Vec<&'a Spec>> {
        // Some(
        //     self.hero_specs[&hero_id]
        //         .iter()
        //         .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        //         .skip(range.start)
        //         .take(range.end - range.start),
        // )
        None
    }

    pub fn set_hero_spec(&mut self, hero_id: usize, spec_name: &str) {
        // self.heroes.get_mut(&hero_id).unwrap().spec = self.hero_specs[&hero_id]
        //     .iter()
        //     .find(|s| s.name == spec_name)
        //     .and_then(|s| Some(s.clone()))
        //     .unwrap();
    }
}

impl<'a> DemoBackend {
    pub fn create_skill(&mut self, name: &str, level: u8, image: &[u8]) {
        // self.skills.push(Skill {
        //     id: self.skills.len(),
        //     name: name.to_string(),
        //     level,
        //     image: RawImage::default(),
        // });
    }

    pub fn modify_skill(&mut self, skill_id: usize, name: &str, level: u8, image: &[u8]) {
        // let skill = self.skills.iter_mut().find(|s| s.id == skill_id).unwrap();
        // skill.name = name.to_string();
        // skill.level = level;
    }

    pub fn get_skill_row_count(&mut self, hero_id: usize, query: &str) -> Option<usize> {
        // Some(
        //     self.skills
        //         .iter()
        //         .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        //         .count(),
        // )
        None
    }

    pub fn get_skill_range(
        &'a mut self,
        hero_id: usize,
        query: &'a str,
        range: &'a Range<usize>,
        // ) -> Option<impl Iterator<Item = &'a Skill>> {
    ) -> Option<Vec<Skill>> {
        // Some(
        //     self.skills
        //         .iter()
        //         .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
        //         .skip(range.start)
        //         .take(range.end - range.start),
        // )
        None
    }

    pub fn set_hero_skill(&mut self, hero_id: usize, idx: usize, skill_id: Option<usize>) {
        // self.heroes.get_mut(&hero_id).unwrap().skills[idx] = if let Some(skill_id) = skill_id {
        //     self.skills
        //         .iter()
        //         .find(|s| s.id == skill_id)
        //         .and_then(|s| Some(s.clone()))
        // } else {
        //     None
        // };
    }
}
