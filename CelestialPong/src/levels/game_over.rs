use macroquad::{
    color::colors, input::is_mouse_button_pressed, math::vec2, text::draw_text, window,
};

use crate::levels::levels::*;

use super::title_screen::TitleScreen;

pub struct GameOver {
    level_parameters: LevelParameters,
    final_score: i32,
}

impl GameOver {
    pub fn game_over(final_score: i32, level_parameters: LevelParameters) -> Level {
        return Level::GameOver(GameOver {
            level_parameters,
            final_score,
        });
    }

    pub fn update(&mut self) -> Level {
        if is_mouse_button_pressed(window::miniquad::MouseButton::Left) {
            return Level::TitleScreen(TitleScreen::new(self.level_parameters));
        }

        Level::None
    }

    pub fn draw(&self) {
        let font_size = 28.;
        let center = vec2(window::screen_width(), window::screen_height()) / 2.;
        draw_text(
            "Congratulation!",
            center.x - 64.,
            center.y - font_size * 2.,
            font_size + 10.,
            colors::GOLD,
        );

        let label = format!("Score : {}", &self.final_score);
        let width = label.len() as f32 * font_size;
        draw_text(
            &label,
            center.x - width / 4.,
            center.y,
            font_size,
            colors::GOLD,
        );
    }
}
