use std::collections::HashMap;
use std::ops::Range;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::hero::{demo_heroes, select_buttons_from_heroes, Hero, HeroSelectButton};
use crate::skill::{demo_skills, Skill};
use crate::spec::{demo_specs, Spec};
use crate::utils::RawImage;

pub struct DemoBackend {
    heroes: HashMap<usize, Hero>,
    specs: Vec<Spec>,
    skills: Vec<Skill>,
    tokio_rt: tokio::runtime::Runtime,
    db_pool: Arc<Mutex<Option<sqlx::pool::Pool<sqlx::postgres::Postgres>>>>,
    repaint_signal: Option<Arc<dyn eframe::epi::RepaintSignal>>,
    status: Arc<Mutex<BackendStatus>>,
    pub messages_receiver: Receiver<String>,
    messages_sender: Sender<String>,
    hero_specs: HashMap<usize, Vec<Spec>>,
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
            heroes: Default::default(),
            specs: Default::default(),
            skills: Default::default(),
            tokio_rt,
            repaint_signal: Default::default(),
            status: Default::default(),
            messages_receiver,
            messages_sender,
            db_pool: Default::default(),
            hero_specs: Default::default(),
        }
    }
}

impl DemoBackend {
    pub fn update(&mut self, frame: &mut eframe::epi::Frame) {
        self.repaint_signal
            .get_or_insert_with(|| frame.repaint_signal());
        if self.db_pool.lock().unwrap().is_none() {
            return;
        }
        // let db = if let Some(db) = &*self.db_pool.lock().unwrap() {
        //     db
        // } else {
        //     return;
        // };

        if self.specs.is_empty() {
            self.specs = demo_specs(frame);
            self.skills = demo_skills(frame);
            for hero in demo_heroes(frame) {
                self.heroes.insert(hero.id, hero);
            }
            for hero in self.heroes.values() {
                self.hero_specs.insert(
                    hero.id,
                    self.specs
                        .iter()
                        .filter(|s| s.class == hero.character.class)
                        .cloned()
                        .collect(),
                );
            }
        }
    }

    pub fn get_status(&self) -> BackendStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn connect_to_db(&mut self) {
        let rt = self.tokio_rt.handle().clone();
        let db_handle = self.db_pool.clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock().unwrap() = BackendStatus::Connecting;
        repaint.request_repaint();

        std::thread::spawn(move || {
            let connect_result = rt.block_on(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect("postgres://homm3:homm3@localhost/homm3_heroes"),
            );
            let db_pool = match connect_result {
                Err(e) => {
                    *status.lock().unwrap() = BackendStatus::NotConnected;
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                    return;
                }
                Ok(db) => db,
            };

            db_handle.lock().unwrap().replace(db_pool);

            *status.lock().unwrap() = BackendStatus::Idle;
            msgs.send("Connected".to_string()).unwrap();
            repaint.request_repaint();
        });
    }

    pub fn create_db(&mut self) {
        let rt = self.tokio_rt.handle().clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock().unwrap() = BackendStatus::Connecting;
        repaint.request_repaint();

        std::thread::spawn(move || {
            let connect_result = rt.block_on(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect("postgres://homm3:homm3@localhost/homm3"),
            );
            let homm3_db = match connect_result {
                Err(e) => {
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                    return;
                }
                Ok(db) => db,
            };
            let q = sqlx::query::<_>("CREATE DATABASE homm3_heroes;").execute(&homm3_db);
            if let Err(e) = rt.block_on(q) {
                *status.lock().unwrap() = BackendStatus::NotConnected;
                msgs.send(e.to_string()).unwrap();
                repaint.request_repaint();
            } else {
                *status.lock().unwrap() = BackendStatus::NotConnected;
                msgs.send("DB Created".to_string()).unwrap();
                repaint.request_repaint();
            }
        });
    }

    pub fn drop_db(&mut self) {
        let rt = self.tokio_rt.handle().clone();
        let db_handle = self.db_pool.clone();
        let msgs = self.messages_sender.clone();
        let repaint = self.repaint_signal.as_ref().unwrap().clone();
        let status = self.status.clone();

        *status.lock().unwrap() = BackendStatus::Connecting;
        msgs.send("Dropping DB...".to_string()).unwrap();
        repaint.request_repaint();

        std::thread::spawn(move || {
            rt.block_on(db_handle.lock().unwrap().as_mut().unwrap().close());
            let connect_result = rt.block_on(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect("postgres://homm3:homm3@localhost/homm3"),
            );
            let homm3_db = match connect_result {
                Err(e) => {
                    msgs.send(e.to_string()).unwrap();
                    repaint.request_repaint();
                    return;
                }
                Ok(db) => db,
            };
            let q = sqlx::query::<_>("DROP DATABASE homm3_heroes;").execute(&homm3_db);
            if let Err(e) = rt.block_on(q) {
                msgs.send(e.to_string()).unwrap();
            }
            *status.lock().unwrap() = BackendStatus::NotConnected;
            msgs.send("DB Dropped successfully".to_string()).unwrap();
            repaint.request_repaint();
        });
    }
}

impl DemoBackend {
    pub fn get_player_heroes(&mut self, player_id: usize) -> Vec<HeroSelectButton> {
        select_buttons_from_heroes(&self.heroes.values().cloned().collect::<Vec<_>>())
    }

    pub fn get_hero(&mut self, hero_id: usize) -> Option<Hero> {
        self.heroes.get(&hero_id).and_then(|h| Some(h.clone()))
    }

    pub fn get_classes(&mut self) -> Option<Vec<String>> {
        Some(vec!["Путешественник".to_string(), "Алхимик".to_string()])
    }

    pub fn set_hero_pskill(&mut self, hero_id: usize, pskill: usize, value: u8) {
        self.heroes.get_mut(&hero_id).unwrap().pskills[pskill] = value;
    }

    pub fn set_hero_xp(&mut self, hero_id: usize, value: u16) {
        self.heroes.get_mut(&hero_id).unwrap().experience = value;
    }

    pub fn set_hero_mana(&mut self, hero_id: usize, current_value: u16, max_value: u16) {
        self.heroes.get_mut(&hero_id).unwrap().mana_current = current_value;
        self.heroes.get_mut(&hero_id).unwrap().mana_max = max_value;
    }

    pub fn set_hero_morale(&mut self, hero_id: usize, value: u8) {
        self.heroes.get_mut(&hero_id).unwrap().morale = value;
    }

    pub fn set_hero_luck(&mut self, hero_id: usize, value: u8) {
        self.heroes.get_mut(&hero_id).unwrap().luck = value;
    }
}

impl<'a> DemoBackend {
    pub fn create_or_modify_spec(&mut self, name: &str, class: &str, image: &[u8]) {}

    pub fn get_specs_row_count(&mut self, hero_id: usize, query: &str) -> Option<usize> {
        Some(
            self.hero_specs[&hero_id]
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
                .count(),
        )
    }

    pub fn get_specs_range(
        &'a mut self,
        hero_id: usize,
        query: &'a str,
        range: &'a Range<usize>,
    ) -> Option<impl Iterator<Item = &'a Spec>> {
        Some(
            self.hero_specs[&hero_id]
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
                .skip(range.start)
                .take(range.end - range.start),
        )
    }

    pub fn set_hero_spec(&mut self, hero_id: usize, spec_name: &str) {
        self.heroes.get_mut(&hero_id).unwrap().spec = self.hero_specs[&hero_id]
            .iter()
            .find(|s| s.name == spec_name)
            .and_then(|s| Some(s.clone()))
            .unwrap();
    }
}

impl<'a> DemoBackend {
    pub fn create_skill(&mut self, name: &str, level: u8, image: &[u8]) {
        self.skills.push(Skill {
            id: self.skills.len(),
            name: name.to_string(),
            level,
            image: RawImage::default(),
        });
    }

    pub fn modify_skill(&mut self, skill_id: usize, name: &str, level: u8, image: &[u8]) {
        let skill = self.skills.iter_mut().find(|s| s.id == skill_id).unwrap();
        skill.name = name.to_string();
        skill.level = level;
    }

    pub fn get_skill_row_count(&mut self, hero_id: usize, query: &str) -> Option<usize> {
        Some(
            self.skills
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
                .count(),
        )
    }

    pub fn get_skill_range(
        &'a mut self,
        hero_id: usize,
        query: &'a str,
        range: &'a Range<usize>,
    ) -> Option<impl Iterator<Item = &'a Skill>> {
        Some(
            self.skills
                .iter()
                .filter(|s| s.name.to_lowercase().contains(&query.to_lowercase()))
                .skip(range.start)
                .take(range.end - range.start),
        )
    }

    pub fn set_hero_skill(&mut self, hero_id: usize, idx: usize, skill_id: Option<usize>) {
        self.heroes.get_mut(&hero_id).unwrap().skills[idx] = if let Some(skill_id) = skill_id {
            self.skills
                .iter()
                .find(|s| s.id == skill_id)
                .and_then(|s| Some(s.clone()))
        } else {
            None
        };
    }
}
