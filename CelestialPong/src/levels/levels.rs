use macroquad::math::Vec2;

use crate::levels::{garden_level::*, sandbox_level::*, title_screen::*};

pub enum Level {
    TitleScreen(TitleScreen),
    SandboxLevel(SandboxLevel),
    GardenLevel(GardenLevel),
    None,
}

impl Level {
    pub fn init(&mut self) {
        match self {
            Level::SandboxLevel(level) => {
                level.init();
            }
            Level::GardenLevel(level) => {
                level.init();
            }
            _ => {}
        }
    }

    pub fn update(&mut self) -> Level {
        match self {
            Level::TitleScreen(title) => title.update(),
            Level::SandboxLevel(level) => level.update(),
            Level::GardenLevel(level) => level.update(),
            _ => Level::None,
        }
    }

    pub fn draw(&self) {
        match self {
            Level::TitleScreen(screen) => {
                screen.draw();
            }
            Level::SandboxLevel(level) => {
                level.draw();
            }
            Level::GardenLevel(level) => {
                level.draw();
            }
            _ => {}
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct LevelParameters {
    pub window_size: [f32; 2],
    pub play_area_size: Vec2,
}
