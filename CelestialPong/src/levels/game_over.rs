use macroquad::{color::colors, text::draw_text};

use crate::levels::levels::*;

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
        Level::None
    }

    pub fn draw(&self) {
        let font_size = 28.;
        draw_text(
            "Congratulation!",
            self.level_parameters.window_size[0] / 2. - 64.,
            self.level_parameters.window_size[1] / 2. - font_size * 2.,
            font_size + 10.,
            colors::GOLD,
        );

        let label = format!("Score : {}", &self.final_score);
        let width = label.len() as f32 * font_size;
        draw_text(
            &label,
            self.level_parameters.window_size[0] / 2. - width / 4.,
            self.level_parameters.window_size[1] / 2.,
            font_size,
            colors::GOLD,
        );
    }
}
