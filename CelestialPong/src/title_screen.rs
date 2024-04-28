use macroquad::{
    input::{is_key_pressed, KeyCode},
    text::{draw_text_ex, TextParams},
};

use crate::{
    levels::{Level, LevelParameters},
    SandboxLevel,
};

#[derive(PartialEq)]
pub struct TitleScreen {
    level_parameters: LevelParameters,
}

impl TitleScreen {
    pub fn new(level_parameters: LevelParameters) -> TitleScreen {
        return TitleScreen { level_parameters };
    }

    pub fn update(&self) -> Level {
        if is_key_pressed(KeyCode::Space) {
            let mut level = SandboxLevel::new(self.level_parameters);
            level.init();
            return Level::SandboxLevel(level);
        }

        return Level::None;
    }

    pub fn draw(&self) {
        let title_label = "Celestial Garden";
        let font_size = 30.;
        let width = title_label.len() as f32 * font_size;
        draw_text_ex(
            "Celestial Garden",
            (&self.level_parameters.window_size[0] - width) / 2.,
            200.,
            TextParams {
                font_size: font_size as u16,
                ..Default::default()
            },
        )
    }
}
