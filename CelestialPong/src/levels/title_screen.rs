use macroquad::{
    color::colors,
    math::{vec2, RectOffset, Vec2},
    text::{draw_text_ex, TextParams},
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
    ui::{root_ui, Skin},
    window,
};

use crate::levels::levels::*;

use super::{sandbox_level::SandboxLevel, tutorial::Tutorial};

pub struct TitleScreen {
    level_parameters: LevelParameters,
    bg_texture: Texture2D,
    button_skin: Skin,
}

impl TitleScreen {
    pub fn new(level_parameters: LevelParameters) -> TitleScreen {
        let button_style = root_ui()
            .style_builder()
            .font_size(20)
            .margin(RectOffset {
                top: 15.,
                right: 15.,
                bottom: 15.,
                left: 15.,
            })
            .color(colors::LIGHTGRAY)
            .color_hovered(colors::GRAY)
            .color_clicked(colors::BEIGE)
            .build();

        let bg_texture = Texture2D::from_file_with_format(
            include_bytes!("..\\..\\textures\\title_screen.png"),
            None,
        );

        return TitleScreen {
            level_parameters,
            bg_texture,
            button_skin: Skin {
                button_style,
                ..root_ui().default_skin()
            },
        };
    }

    pub fn update(&self) -> Level {
        root_ui().push_skin(&self.button_skin.clone());
        let sky_level = root_ui().button(
            vec2(
                window::screen_width() / 2. - 20.,
                window::screen_height() / 2. + 160.,
            ),
            "Start",
        );
        let sandbox_level = root_ui().button(
            vec2(window::screen_width() - 100., window::screen_height() - 60.),
            "Sandbox",
        );
        root_ui().pop_skin();

        if sky_level {
            return Level::Tutorial(Tutorial::new(self.level_parameters));
        } else if sandbox_level {
            return Level::SandboxLevel(SandboxLevel::new(self.level_parameters));
        }

        return Level::None;
    }

    pub fn draw(&self) {
        let center = vec2(window::screen_width(), window::screen_height()) / 2.;

        draw_texture_ex(
            &self.bg_texture,
            0.0,
            0.0,
            colors::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(window::screen_width(), window::screen_height())),
                ..Default::default()
            },
        );

        draw_text_ex(
            "Game by AntonMakesGames",
            30.,
            center.y * 2. - 25.,
            TextParams {
                font_size: 28,
                ..Default::default()
            },
        );

        draw_text_ex(
            "Music by Klaim!",
            40.,
            center.y * 2. - 5.,
            TextParams {
                font_size: 28,
                ..Default::default()
            },
        );
    }
}
