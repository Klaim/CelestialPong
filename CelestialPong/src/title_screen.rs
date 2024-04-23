use macroquad::{
    input::{is_key_pressed, KeyCode},
    math::Vec2,
    text::{draw_text_ex, TextParams},
};

use crate::{level::Level, SandboxLevel};

#[derive(PartialEq)]
pub struct TitleScreen {
    window_size: [f32; 2],
    play_area_size: Vec2,
}

impl TitleScreen {
    pub fn new(window_size: [f32; 2], play_area_size: Vec2) -> TitleScreen {
        return TitleScreen {
            window_size,
            play_area_size,
        };
    }

    pub fn update(&self) -> Level {
        if is_key_pressed(KeyCode::Space) {
            let mut level = SandboxLevel::new(self.window_size, self.play_area_size);
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
            (&self.window_size[0] - width) / 2.,
            200.,
            TextParams {
                font_size: font_size as u16,
                ..Default::default()
            },
        )
    }
}
