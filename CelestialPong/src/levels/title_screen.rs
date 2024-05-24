use macroquad::{
    color::colors,
    math::{vec2, RectOffset},
    text::{draw_text_ex, TextParams},
    ui::{root_ui, Skin},
    window,
};

use crate::levels::levels::*;

use super::{garden_level::GardenLevel, sandbox_level::SandboxLevel};

pub struct TitleScreen {
    level_parameters: LevelParameters,

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

        return TitleScreen {
            level_parameters,
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
                window::screen_height() / 2.,
            ),
            "Start",
        );
        let sandbox_level = root_ui().button(
            vec2(window::screen_width() - 100., window::screen_height() - 60.),
            "Sandbox",
        );
        root_ui().pop_skin();

        if sky_level {
            return Level::GardenLevel(GardenLevel::new(self.level_parameters));
        } else if sandbox_level {
            return Level::SandboxLevel(SandboxLevel::new(self.level_parameters));
        }

        return Level::None;
    }

    pub fn draw(&self) {
        let font_size = 64.;
        let center = vec2(window::screen_width(), window::screen_height()) / 2.;
        draw_text_ex(
            "Celestial Garden",
            center.x - 250.,
            center.y - 200.,
            TextParams {
                font_size: font_size as u16,
                ..Default::default()
            },
        );

        draw_text_ex(
            "Remove all the bad seed hanging around the planet",
            center.x - 240.,
            center.y - 100.,
            TextParams {
                font_size: 24,
                ..Default::default()
            },
        );

        draw_text_ex(
            "the more carefull you are, the higher your score!",
            center.x - 200.,
            center.y - 80.,
            TextParams {
                font_size: 24,
                ..Default::default()
            },
        );

        draw_text_ex(
            "By AntonMakesGames",
            10.,
            center.y * 2. - 5.,
            TextParams {
                ..Default::default()
            },
        );
    }
}