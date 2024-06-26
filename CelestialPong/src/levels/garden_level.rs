// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

use macroquad::{
    color::{self, colors, hsl_to_rgb},
    prelude::*,
    rand::{srand, RandomRange},
};

use crate::{
    levels::{levels::*, title_screen::*},
    simulation::{ball::*, gravity::*, quad_tree::*},
    visual::radial_gradiant::get_radial_gradient_texture,
};

use crate::{simulation::quad_tree, SIMULATION_DT};

use super::game_over::GameOver;

const NB_BALLS: usize = 300;
const BALL_RADII: f32 = 7.;
const BALL_MASS: f32 = 40.;

const NB_BAD_BALLS: usize = 20;
const NB_SEED: usize = 15;

const BODY_MASS: f32 = 10000000.;
// const BODY_BOUNCYNESS: f32 = 0.9;

const ORBIT_TRAP: f32 = 10.0;
const ORBIT_TRAP_SIZE: f32 = 9.;

const MIN_START_ORBIT: f32 = 210.;
const MAX_START_ORBIT: f32 = 351.;

const TRACE_SIZE: usize = 5000;

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

struct SeededFlower {
    position: Vec2,
    rotation: f32,
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

        let ball_type = match index < NB_BAD_BALLS {
            true => BallType::BadBall,
            false => match index < NB_BAD_BALLS + NB_SEED {
                true => BallType::GoodBall,
                false => BallType::Ball,
            },
        };

        let color = match ball_type {
            BallType::BadBall | BallType::GoodBall => WHITE,
            _ => hsl_to_rgb(
                RandomRange::gen_range(0., 1.),
                RandomRange::gen_range(0.45, 0.95),
                RandomRange::gen_range(0.65, 0.99),
            ),
        };

        let radius = match ball_type {
            BallType::BadBall => BALL_RADII * 1.3,
            BallType::GoodBall => BALL_RADII * 0.8,
            _ => BALL_RADII,
        };

        let mut ball = Ball::new(
            position,
            Vec2::ZERO,
            radius,
            BALL_MASS,
            color,
            RandomRange::gen_range(-std::f32::consts::PI, std::f32::consts::PI),
            RandomRange::gen_range(-std::f32::consts::PI * 2., std::f32::consts::PI * 2.),
            ball_type,
        );

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

    seeded_flowers: Vec<SeededFlower>,

    kill_distance_squared: f32,
    level_parameters: LevelParameters,
    background: Texture2D,

    body_texture: Texture2D,
    ball_texture: Texture2D,
    bad_ball_texture: Texture2D,
    seed_texture: Texture2D,
    seeded_flower_texture: Texture2D,
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
            seeded_flowers: Vec::new(),
            background,
            body_texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\planet2.png"),
                None,
            ),

            ball_texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\flower_white.png"),
                None,
            ),

            bad_ball_texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\spike_v2.png"),
                None,
            ),

            seed_texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\seed_v2.png"),
                None,
            ),

            seeded_flower_texture: Texture2D::from_file_with_format(
                include_bytes!("..\\..\\textures\\flower_sproute.png"),
                None,
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
            0.0,
            0.0,
            BallType::Body,
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
            self.seeded_flowers.clear();
        }

        if is_key_pressed(KeyCode::Backspace) {
            let title = TitleScreen::new(self.level_parameters);
            return Level::TitleScreen(title);
        }

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
                } else if ball.ball_type == BallType::BadBall {
                    if ball.position.x.abs() > self.level_parameters.window_size[0] / 2.
                        || ball.position.y.abs() > self.level_parameters.window_size[1] / 2.
                    {
                        self.balls_marked_for_delete.push(index);
                    }
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
                        // DELETE
                        if !self.balls_marked_for_delete.contains(&near.payload) {
                            self.balls_marked_for_delete.push(near.payload);
                            if ball.ball_type == BallType::GoodBall {
                                let direction = (body.position - ball.position).normalize();
                                self.seeded_flowers.push(SeededFlower {
                                    position: ball.position + direction * ball.radius * -1.5,
                                    rotation: -direction.angle_between(vec2(0.0, 1.0))
                                        + RandomRange::gen_range(-0.22, 0.22),
                                });
                            }
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

        if is_mouse_button_pressed(MouseButton::Left) {
            let ball_vel = (mouse_pos - self.player.position).normalize() * 7.;
            let ball = Ball::new(
                self.player.position,
                ball_vel,
                BALL_RADII * 1.2,
                BALL_MASS * 2.,
                colors::BLUE,
                0.0,
                0.0,
                BallType::Projectil,
            );

            self.balls.push(ball);
        }

        if !self
            .balls
            .iter()
            .any(|ball| ball.ball_type == BallType::BadBall)
        {
            let score = self
                .balls
                .iter()
                .filter(|ball| ball.ball_type == BallType::Ball)
                .count()
                + self.seeded_flowers.len() * 10;
            return GameOver::game_over(score as i32, self.level_parameters);
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

        // Draw trace objects
        // for trace in self.traces {
        //     draw_circle(trace.x, trace.y, 1., colors::BLUE);
        // }

        self.player.draw();

        for ball in &self.balls {
            let texture = match ball.ball_type {
                BallType::BadBall => Some(&self.bad_ball_texture),
                BallType::Ball => Some(&self.ball_texture),
                BallType::GoodBall => Some(&self.seed_texture),
                _ => None,
            };
            ball.draw(texture);
            if ball.ball_type == BallType::Ball {
                draw_circle(
                    ball.position.x,
                    ball.position.y,
                    ball.radius / 4.,
                    hsl_to_rgb(0.65, 0.80, 0.65),
                );
            }
        }

        for body in &self.static_bodies {
            body.draw(Some(&self.body_texture));
        }

        for flower in &self.seeded_flowers {
            draw_texture_ex(
                &self.seeded_flower_texture,
                flower.position.x - 16.,
                flower.position.y - 16.,
                colors::WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(20., 40.)),
                    rotation: flower.rotation,
                    ..Default::default()
                },
            );
        }

        // quad_tree.debug_draw();
        set_default_camera();
    }
}
