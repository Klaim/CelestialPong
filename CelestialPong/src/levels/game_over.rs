use macroquad::{
    color::colors, input::is_mouse_button_pressed, math::vec2, prelude::*, text::draw_text,
    texture::Texture2D, window,
};

use crate::levels::levels::*;

use super::title_screen::TitleScreen;

pub struct GameOver {
    level_parameters: LevelParameters,
    final_score: i32,
    texture: Texture2D,
}

impl GameOver {
    pub fn game_over(final_score: i32, level_parameters: LevelParameters) -> Level {
        return Level::GameOver(GameOver {
            level_parameters,
            texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\end_screen.png"),
                None,
            ),
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
        let font_size = 42.;
        let center = vec2(window::screen_width(), window::screen_height()) / 2.;

        draw_texture_ex(
            &self.texture,
            0.0,
            center.y / 2.,
            colors::WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(window::screen_width(), window::screen_width() / 2.0)),
                ..Default::default()
            },
        );

        let label = format!("{}", &self.final_score);
        draw_text(
            &label,
            center.x + center.x * 0.5,
            center.y * 0.86,
            font_size,
            colors::WHITE,
        );
    }
}
