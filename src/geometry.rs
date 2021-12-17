use eframe::egui::{self, Vec2};
use egui::{Pos2, Rect};

macro_rules! pos {
    ($($name:ident, $a_x:expr, $a_y:expr, $width:expr, $height:expr),+) => {
        $(
        pub const $name: Rect =
            Rect::from_min_max(Pos2 { x: $a_x as f32, y: $a_y as f32}, Pos2 { x: ($width+$a_x) as f32, y: ($height+$a_y) as f32});
        )+
    };
}

pos! {
    H_PORTRAIT, 19, 19, 58, 64,
    H_NAME, 81, 20, 217, 34,
    H_CLASS, 81, 50, 218, 23,
    H_SWITCHER_PORTRAIT, 612, 86, 48, 32,
    SPEC_IMAGE, 18, 180, 44, 44,
    PSKILL_IMAGE, 32, 111, 42, 42,
    PSKILL_NAME, 32, 92, 42, 13,
    PSKILL_VALUE, 32, 158, 42, 13,
    SKILL_IMAGE, 18, 228, 44, 44,
    SKILL_TEXT, 67, 232, 89, 38,
    SKILL_BOX, 18, 228, 139, 46,
    MORALE_IMAGE, 182, 184, 42, 38,
    LUCK_IMAGE, 240, 184, 42, 38,
    INFO_BOX, 8, 558, 656, 20,
    FLAG_IMAGE, 606, 8, 58, 64
}

pub const WINDOW_SIZE: Vec2 = Vec2 { x: 672., y: 586. };
pub const H_SWITCHER_PORTRAIT_OFFSET: Vec2 = Vec2 { x: 0., y: 54. };
pub const PSKILL_OFFSET: Vec2 = Vec2 { x: 70., y: 0. };
pub const SKILL_OFFSET_H: Vec2 = Vec2 { x: 142., y: 0. };
pub const SKILL_OFFSET_V: Vec2 = Vec2 { x: 0., y: 48. };
