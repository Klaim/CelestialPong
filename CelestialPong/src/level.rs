use crate::{title_screen::TitleScreen, SandboxLevel};

pub enum Level {
    TitleScreen(TitleScreen),
    SandboxLevel(SandboxLevel),
    None,
}

impl Level {
    pub fn init(&mut self) {
        match self {
            Level::SandboxLevel(level) => {
                level.init();
            }
            _ => {}
        }
    }

    pub fn update(&mut self) -> Level {
        match self {
            Level::TitleScreen(title) => title.update(),
            Level::SandboxLevel(level) => level.update(),
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
            _ => {}
        }
    }
}
