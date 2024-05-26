// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

mod levels;
mod simulation;
mod visual;

use levels::levels::{Level, LevelParameters};
use levels::title_screen::TitleScreen;

use macroquad::{prelude::*, window};

const WINDOW_SIZE: [f32; 2] = [1000., 1000.];
const FPS_FRAMES: usize = 100;

const SIMULATION_DT: f32 = 1. / 240.;

fn window_config() -> Conf {
    Conf {
        window_title: "Celestial pong".to_owned(),
        // window_width: WINDOW_SIZE[0] as i32,
        // window_height: WINDOW_SIZE[1] as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let play_area_size = Vec2::new(window::screen_width(), window::screen_height());
    let level_parameters = LevelParameters {
        play_area_size,
        window_size: WINDOW_SIZE,
    };

    let mut level = Level::TitleScreen(TitleScreen::new(level_parameters));
    level.init();

    let mut frame_per_frame: usize = 1;
    let mut fps: [f32; FPS_FRAMES] = [0.; FPS_FRAMES];
    let mut fps_index: usize = 0;

    loop {
        let frame_start = get_time();
        if is_key_pressed(KeyCode::Up) {
            frame_per_frame = frame_per_frame + 1;
        }

        if is_key_pressed(KeyCode::Down) {
            frame_per_frame = (frame_per_frame - 1).max(1);
        }

        let dt = get_frame_time();
        fps[fps_index] = dt;
        fps_index = (fps_index + 1) % FPS_FRAMES;

        let mut next_level = Level::None;
        for _frame in 0..frame_per_frame {
            next_level = level.update();
        }

        level.draw();

        set_default_camera();
        {
            let mean_fps = Iterator::sum::<f32>(fps.iter()) / FPS_FRAMES as f32;
            draw_text_ex(
                &format!("fps : {}", 1. / mean_fps),
                32.,
                32.,
                TextParams {
                    font_size: 15,
                    ..Default::default()
                },
            );

            draw_text_ex(
                &format!("Simulation speed : {}", frame_per_frame),
                32.,
                50.,
                TextParams {
                    font_size: 15,
                    ..Default::default()
                },
            );
        }

        level = match next_level {
            Level::None => level,
            _ => {
                next_level.init();
                next_level
            }
        };

        let elapsed_time = get_time() - frame_start;
        let time_since_framerate = (1. / 61.) - elapsed_time;

        if time_since_framerate > 0. {
            std::thread::sleep(std::time::Duration::from_millis((time_since_framerate * 1000.) as u64));
        }

        next_frame().await
    }
}
