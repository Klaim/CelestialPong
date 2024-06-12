use macroquad::{
    color::colors,
    input::is_mouse_button_pressed,
    math::{vec2, Vec2},
    prelude::*,
    texture::{DrawTextureParams, Texture2D},
    window,
};

use super::{
    garden_level::GardenLevel,
    levels::{Level, LevelParameters},
};

pub struct Tutorial {
    level_parameters: LevelParameters,
    texture: Texture2D,
}

impl Tutorial {
    pub fn new(level_parameters: LevelParameters) -> Tutorial {
        Tutorial {
            level_parameters,
            texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\tutorial_screen.png"),
                None,
            ),
        }
    }

    pub fn update(&mut self) -> Level {
        if is_mouse_button_pressed(window::miniquad::MouseButton::Left) {
            return Level::GardenLevel(GardenLevel::new(self.level_parameters));
        }

        Level::None
    }

    pub fn draw(&self) {
        clear_background(colors::BLACK);

        let margin = 80.0;

        draw_texture_ex(
            &self.texture,
            margin,
            margin,
            colors::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    window::screen_width() - margin * 2.0,
                    window::screen_height() - margin * 2.0,
                )),
                ..Default::default()
            },
        );
    }
}
