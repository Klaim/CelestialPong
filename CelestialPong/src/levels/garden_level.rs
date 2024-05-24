// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

use macroquad::{
    color::{self, colors},
    prelude::*,
    rand::{srand, RandomRange},
    ui::{root_ui, Skin},
    window,
};

use crate::{
    levels::{levels::*, title_screen::*},
    simulation::{
        ball::*,
        gravity::*,
        quad_tree::{Rect, *},
    },
    visual::{
        radial_gradiant::get_radial_gradient_texture,
        ui_textures::{get_anti_clockwise_skin, get_clockwise_skin},
    },
};

use crate::{simulation::quad_tree, SIMULATION_DT};

use super::game_over::GameOver;

const NB_BALLS: usize = 300;
const BALL_RADII: f32 = 4.;
const BALL_MASS: f32 = 40.;

const BODY_MASS: f32 = 10000000.;
// const BODY_BOUNCYNESS: f32 = 0.9;

const ORBIT_TRAP: f32 = 10.0;
const ORBIT_TRAP_SIZE: f32 = 9.;

const MIN_START_ORBIT: f32 = 220.;
const MAX_START_ORBIT: f32 = 321.;

const TRACE_SIZE: usize = 1000;

const BAD_BALL_COLOR: Color = Color {
    r: 0.9,
    g: 0.1,
    b: 0.1,
    a: 1.,
};

struct Player {
    position: Vec2,
    orientation: f32,
    orbiting_center: Vec2,
    orbiting_radius: f32,
    azimut: f32,
    azimut_speed: f32,
}

impl Player {
    pub fn update(&mut self, dt: f32) {
        self.azimut = self.azimut + self.azimut_speed * dt;
        self.position = self.orbiting_center
            + Vec2::from_angle(self.azimut).rotate(Vec2::X) * self.orbiting_radius;
    }

    pub fn draw(&self) {
        draw_poly(
            self.position.x,
            self.position.y,
            3,
            10.,
            self.orientation,
            colors::GOLD,
        );
    }
}

fn random_orbital_pos(center: Vec2, min_radius: f32, max_radius: f32) -> Vec2 {
    let angle = RandomRange::gen_range(0., std::f32::consts::PI * 2.);
    let result = Vec2::from((angle.cos(), angle.sin()));
    let rad = RandomRange::gen_range(min_radius, max_radius);
    let result = center + result * rad;
    return result;
}

fn reset_balls(balls: &mut Vec<Ball>, static_bodies: &Vec<Ball>) {
    balls.clear();

    for index in 0..NB_BALLS {
        let position =
            random_orbital_pos(static_bodies[0].position, MIN_START_ORBIT, MAX_START_ORBIT);

        let color = match index < NB_BALLS / 10 {
            true => BAD_BALL_COLOR,
            false => Color {
                r: 0.75,
                g: 0.75,
                b: 0.9,
                a: 1.,
            },
        };

        let mut ball = Ball::new(position, Vec2::ZERO, BALL_RADII, BALL_MASS, color);

        let ball_speed = get_orbital_velocity(&ball, &static_bodies[0]);

        ball.set_velocity(ball_speed, SIMULATION_DT);
        balls.push(ball);
    }
}

pub struct GardenLevel {
    paused: bool,
    balls: Vec<Ball>,
    static_bodies: Vec<Ball>,
    tree_area: quad_tree::Rect,
    quad_tree: QuadTree,

    player: Player,

    main_camera: Camera2D,
    collided_balls: Vec<usize>,
    balls_marked_for_delete: Vec<usize>,
    traces: [Vec2; TRACE_SIZE],
    trace_index: usize,

    kill_distance_squared: f32,
    level_parameters: LevelParameters,
    background: Texture2D,

    anticlockwise_btn_rect: Rect,
    anticlockwise_skin: Skin,
    clockwise_btn_rect: Rect,
    clockwise_skin: Skin,
}

enum UIActions {
    Clockwise,
    Anticlockwise,
    None,
}

impl GardenLevel {
    pub fn new(level_parameters: LevelParameters) -> GardenLevel {
        let tree_area = quad_tree::Rect::new(
            0.,
            0.,
            level_parameters.play_area_size.x * 4.,
            level_parameters.play_area_size.x * 4.,
        );

        let background = get_radial_gradient_texture(
            level_parameters.window_size[0] as u32,
            level_parameters.window_size[1] as u32,
            colors::BLUE,
        );

        let center_x = window::screen_width() / 2.;
        let bottom = window::screen_height() - 60.;
        let btn_size = 50.;
        let anticlockwise_btn_rect = Rect::new(center_x + 55., bottom, btn_size, btn_size);

        let clockwise_btn_rect = Rect::new(center_x - 55., bottom, btn_size, btn_size);

        return GardenLevel {
            paused: false,
            balls: Vec::with_capacity(NB_BALLS),
            static_bodies: Vec::new(),
            tree_area,
            quad_tree: QuadTree::new(tree_area),

            main_camera: Camera2D {
                zoom: Vec2::from((
                    2. / level_parameters.window_size[0],
                    2. / level_parameters.window_size[1],
                )),
                ..Default::default()
            },

            player: Player {
                position: Vec2::new(300., 300.),
                orientation: 0.,
                orbiting_center: vec2(0., 0.),
                orbiting_radius: 400.,
                azimut: 0.,
                azimut_speed: -0.15,
            },

            collided_balls: Vec::with_capacity(NB_BALLS),
            balls_marked_for_delete: Vec::with_capacity(NB_BALLS),
            traces: [Vec2::ZERO; TRACE_SIZE],
            trace_index: 0,
            level_parameters,
            kill_distance_squared: f32::powf(
                level_parameters.window_size[0] * f32::sqrt(2.) / 2.,
                2.,
            ),
            background,
            anticlockwise_btn_rect,
            anticlockwise_skin: get_anti_clockwise_skin(
                anticlockwise_btn_rect.half_width * 2.,
                anticlockwise_btn_rect.half_height * 2.,
            ),
            clockwise_btn_rect,
            clockwise_skin: get_clockwise_skin(
                clockwise_btn_rect.half_width * 2.,
                clockwise_btn_rect.half_height * 2.,
            ),
        };
    }

    pub fn init(&mut self) {
        self.static_bodies.push(Ball::new(
            Vec2::new(0., 0.),
            Vec2::ZERO,
            90.,
            BODY_MASS,
            color::WHITE,
        ));

        reset_balls(&mut self.balls, &self.static_bodies);
    }

    pub fn update(&mut self) -> Level {
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }

        if is_key_down(KeyCode::R) {
            srand(1);
            reset_balls(&mut self.balls, &self.static_bodies);
        }

        if is_key_pressed(KeyCode::Backspace) {
            let title = TitleScreen::new(self.level_parameters);
            return Level::TitleScreen(title);
        }

        let ui_action = self.update_ui();

        let dt = SIMULATION_DT;

        if !self.paused {
            self.quad_tree = QuadTree::new(self.tree_area);
            // Updating ball position
            self.collided_balls.clear();
            let nb_balls = self.balls.len();
            for index in 0..nb_balls {
                let ball = self.balls.get(index).unwrap();
                self.quad_tree.add(QuadTreeEntry::new(ball.position, index));
                let mut local_force = Vec2::ZERO;

                // Comuting gravity
                for body in &self.static_bodies {
                    local_force = local_force + get_gravity_force(ball, body)
                }

                // Trapping ball in the nearest body
                match self.static_bodies.iter().min_by(|body, other| {
                    (body.position - ball.position)
                        .length_squared()
                        .total_cmp(&(other.position - ball.position).length_squared())
                }) {
                    Some(closest_body) => {
                        let ideal_velocity = get_orbital_velocity(ball, closest_body);
                        let delta = if ideal_velocity.dot(ball.velocity) > 0. {
                            ideal_velocity - ball.velocity
                        } else {
                            ideal_velocity * -1. - ball.velocity
                        };
                        // If the ball velocity differ from the ideal orbit, nudge the ball toward that velocity
                        if delta.length_squared() > ORBIT_TRAP_SIZE {
                            local_force = local_force + (delta / delta.length()) * ORBIT_TRAP;
                        }
                    }
                    _ => {}
                }

                let ball = self.balls.get_mut(index).unwrap();
                // ball.update(dt, local_force);
                ball.update_verlet(dt, local_force);

                // Recode previous positions
                self.traces[self.trace_index] = ball.position;
                self.trace_index = (self.trace_index + 1) % self.traces.len();

                // Delete balls that have gone too far
                if ball.position.length_squared() > self.kill_distance_squared {
                    self.balls_marked_for_delete.push(index);
                }
            }

            // Colliding balls
            for index in 0..self.balls.len() {
                // Has ball already collided this frame
                if self.collided_balls.iter().any(|c| c == &index) {
                    continue;
                }

                let zone_check = self.balls[index].get_collision_area();
                let mut near_balls = Vec::new();
                self.quad_tree.query_entries(&zone_check, &mut near_balls);
                for entry in near_balls {
                    if entry.payload == index
                        || self.collided_balls.iter().any(|c| c == &entry.payload)
                    {
                        continue;
                    }

                    let other_ball_index = entry.payload;

                    if self.balls[index].check_collision(&self.balls[other_ball_index]) {
                        if index > other_ball_index {
                            let (left, right) = self.balls.split_at_mut(index);
                            right[0].collide(&mut left[other_ball_index], dt);
                        } else {
                            let (left, right) = self.balls.split_at_mut(other_ball_index);
                            right[0].collide(&mut left[index], dt);
                        }

                        self.collided_balls.push(index);
                        self.collided_balls.push(other_ball_index);
                    }
                }
            }

            // Bounce of static bodies
            for body in self.static_bodies.iter_mut() {
                let query = body.get_collision_area();
                let mut near_objects = Vec::new();
                self.quad_tree.query_entries(&query, &mut near_objects);
                for near in near_objects {
                    let ball = self.balls.get_mut(near.payload).unwrap();
                    if body.check_collision(&ball) {
                        // BOUNCE
                        // let delta = ball.position - body.position;
                        // if delta.dot(ball.velocity) < 0.
                        //     && ball.velocity.length_squared() > 0.001
                        // {
                        //     let delta = delta.normalize();
                        //     ball.position = body.position + delta * (body.radius + ball.radius);
                        //     ball.set_velocity(
                        //         (ball.velocity - 2. * delta.dot(ball.velocity) * delta)
                        //             * BODY_BOUNCYNESS,
                        //         dt,
                        //     );
                        // }

                        // DELETE
                        if !self.balls_marked_for_delete.contains(&near.payload) {
                            self.balls_marked_for_delete.push(near.payload);
                        }
                    }
                }
            }

            self.balls_marked_for_delete.sort_unstable();
            for index in self.balls_marked_for_delete.iter().rev() {
                self.balls.remove(*index);
            }

            self.balls_marked_for_delete.clear();

            self.player.update(dt);
        }

        let (spx, spy) = mouse_position();
        let mouse_pos = Vec2::new(spx, spy);
        let mouse_pos = self.main_camera.screen_to_world(mouse_pos);

        let player_to_mouse = mouse_pos - self.player.position;
        let player_orientation =
            -player_to_mouse.normalize().angle_between(Vec2::X) / std::f32::consts::PI * 180.;
        self.player.orientation = player_orientation;

        let dist_check = BALL_RADII * BALL_RADII * 10.;
        let mut near_balls = Vec::new();
        self.quad_tree.query_entries(
            &quad_tree::Rect::new(mouse_pos.x, mouse_pos.y, dist_check, dist_check),
            &mut near_balls,
        );

        match ui_action {
            UIActions::None => {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let ball_vel = (mouse_pos - self.player.position).normalize() * 7.;
                    let ball = Ball::new(
                        self.player.position,
                        ball_vel,
                        BALL_RADII * 2.,
                        BALL_MASS * 2.,
                        colors::BLUE,
                    );

                    self.balls.push(ball);
                }
            }
            UIActions::Anticlockwise => {
                self.player.azimut_speed = -1.;
            }
            UIActions::Clockwise => {
                self.player.azimut_speed = 1.;
            }
        }

        if !self.balls.iter().any(|ball| ball.color == BAD_BALL_COLOR) {
            return GameOver::game_over(self.balls.len() as i32, self.level_parameters);
        }

        return Level::None;
    }

    pub fn draw(&self) {
        draw_texture_ex(
            &self.background,
            0.,
            0.,
            colors::WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        set_camera(&self.main_camera);

        self.player.draw();

        for ball in &self.balls {
            ball.draw();
        }

        for body in &self.static_bodies {
            body.draw();
        }

        // quad_tree.debug_draw();

        // Draw trace objects
        // for trace in traces {
        //     draw_circle(trace.x, trace.y, 1., colors::BLUE);
        // }

        set_default_camera();
    }

    fn update_ui(&self) -> UIActions {
        // root_ui().push_skin(&self.anticlockwise_skin);
        // root_ui().button(
        //     vec2(
        //         self.anticlockwise_btn_rect.left,
        //         self.anticlockwise_btn_rect.up,
        //     ),
        //     "",
        // );
        // root_ui().pop_skin();

        // root_ui().push_skin(&self.clockwise_skin);
        // root_ui().button(
        //     vec2(self.clockwise_btn_rect.left, self.anticlockwise_btn_rect.up),
        //     "",
        // );
        // root_ui().pop_skin();

        // let mouse_pos = Vec2::from(mouse_position());
        // if is_mouse_button_down(MouseButton::Left) {
        //     if self.anticlockwise_btn_rect.contains(mouse_pos) {
        //         return UIActions::Anticlockwise;
        //     }

        //     if self.clockwise_btn_rect.contains(mouse_pos) {
        //         return UIActions::Clockwise;
        //     }
        // }

        return UIActions::None;
    }
}
