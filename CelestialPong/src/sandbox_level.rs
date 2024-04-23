// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

use macroquad::{
    color::{self, colors},
    prelude::*,
    rand::{srand, RandomRange},
};

use crate::{
    ball::*,
    gravity::{damping, get_gravity_force, get_orbital_velocity},
    level::*,
};
use crate::{
    quad_tree::{self, *},
    SIMULATION_DT,
};

const NB_BALLS: usize = 220;
const RADII: f32 = 3.;
const BALL_MASS: f32 = 40.;

const BODY_MASS: f32 = 5000000.;
// const BODY_BOUNCYNESS: f32 = 0.9;

const ORBIT_TRAP: f32 = 10.0;
const ORBIT_TRAP_SIZE: f32 = RADII * RADII;

const MIN_START_ORBIT: f32 = 290.;
const MAX_START_ORBIT: f32 = 301.;

const TRACE_SIZE: usize = 1000;

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

        let color = match index < NB_BALLS / 2 {
            true => Color {
                r: 0.9,
                g: 0.16,
                b: 0.16,
                a: 1.,
            },
            false => Color {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.,
            },
        };

        let mut ball = Ball::new(position, Vec2::ZERO, RADII, BALL_MASS, color);

        let ball_speed = get_orbital_velocity(&ball, &static_bodies[0]);

        ball.set_velocity(ball_speed, SIMULATION_DT);
        balls.push(ball);
    }
}

pub struct SandboxLevel {
    paused: bool,
    drawing_enabled: bool,
    balls: Vec<Ball>,
    static_bodies: Vec<Ball>,
    tree_area: quad_tree::Rect,
    quad_tree: QuadTree,

    main_camera: Camera2D,
    collided_balls: Vec<usize>,
    balls_marked_for_delete: Vec<usize>,
    selected_ball: Option<usize>,
    traces: [Vec2; TRACE_SIZE],
    trace_index: usize,
    ball_under: Option<usize>,
}

impl SandboxLevel {
    pub fn new(window_size: [f32; 2], play_area_size: Vec2) -> SandboxLevel {
        let tree_area = quad_tree::Rect::new(0., 0., play_area_size.x * 4., play_area_size.x * 4.);
        return SandboxLevel {
            paused: false,
            drawing_enabled: true,
            balls: Vec::with_capacity(NB_BALLS),
            static_bodies: Vec::new(),
            tree_area,
            quad_tree: QuadTree::new(tree_area),

            main_camera: Camera2D {
                zoom: Vec2::from((2. / window_size[0], 2. / window_size[1])),
                ..Default::default()
            },

            collided_balls: Vec::with_capacity(NB_BALLS),
            balls_marked_for_delete: Vec::with_capacity(NB_BALLS),
            selected_ball: None,
            traces: [Vec2::ZERO; TRACE_SIZE],
            trace_index: 0,
            ball_under: None,
        };
    }

    pub fn init(&mut self) {
        self.static_bodies.push(Ball::new(
            Vec2::new(0., 0.),
            Vec2::ZERO,
            30.,
            BODY_MASS,
            color::WHITE,
        ));

        reset_balls(&mut self.balls, &self.static_bodies);
    }

    pub fn update(&mut self) -> Level {
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }

        if is_key_pressed(KeyCode::V) {
            self.drawing_enabled = !self.drawing_enabled;
        }

        if is_key_down(KeyCode::S) {
            for ball in &mut self.balls {
                ball.set_velocity(ball.velocity * 0.5, SIMULATION_DT);
            }
        }

        if is_key_down(KeyCode::R) {
            self.selected_ball = None;
            self.paused = true;
            srand(1);
            reset_balls(&mut self.balls, &self.static_bodies);
        }

        if is_key_down(KeyCode::O) {
            for ball in &mut self.balls {
                ball.set_velocity(
                    get_orbital_velocity(ball, &self.static_bodies[0]),
                    SIMULATION_DT,
                );
            }
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
                if self.selected_ball == None || self.selected_ball.unwrap() != index {
                    for body in &self.static_bodies {
                        local_force = local_force + get_gravity_force(ball, body)
                    }
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
                match self.selected_ball {
                    Some(selected) => {
                        if index == &selected {
                            self.selected_ball = None;
                        } else if index < &selected {
                            self.selected_ball = Some(selected - 1);
                        }
                    }
                    None => {}
                }
            }

            self.balls_marked_for_delete.clear();
        }

        let (spx, spy) = mouse_position();
        let mouse_pos = Vec2::new(spx, spy);
        let mouse_pos = self.main_camera.screen_to_world(mouse_pos);
        let dist_check = RADII * RADII * 10.;
        let mut near_balls = Vec::new();
        self.quad_tree.query_entries(
            &quad_tree::Rect::new(mouse_pos.x, mouse_pos.y, dist_check, dist_check),
            &mut near_balls,
        );

        self.ball_under = near_balls
            .into_iter()
            .find(|b| (self.balls[b.payload].position - mouse_pos).length_squared() < dist_check)
            .map(|b| b.payload);

        if is_mouse_button_pressed(MouseButton::Left) {
            match self.ball_under {
                Some(entry) => {
                    self.selected_ball = Some(entry);
                }
                _ => {}
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            self.selected_ball = None;
        }

        match self.selected_ball {
            Some(ball_index) => {
                let ball = self.balls.get_mut(ball_index).unwrap();
                let force = damping(ball.position, mouse_pos, dt, 0.05 * dt);
                ball.set_velocity(force, dt);
            }
            _ => {}
        }

        return Level::None;
    }

    pub fn draw(&self) {
        if self.drawing_enabled {
            set_camera(&self.main_camera);

            for ball in &self.balls {
                ball.draw();

                // ball.get_collision_area().debug_draw(1., ball.color);

                // Draw ideal orbit
                let mut c = ball.color;
                c.r = c.r - 10.;
                // draw_poly_lines(
                //     static_bodies[0].position.x,
                //     static_bodies[0].position.y,
                //     100,
                //     (static_bodies[0].position - ball.position).length(),
                //     0.,
                //     1.,
                //     c,
                // );

                // draw sphere of influence
                // let influence = get_gravity_radius_over_threshold(ball.mass, 0.001);
                // draw_circle_lines(
                //     ball.position.x,
                //     ball.position.y,
                //     influence,
                //     1.,
                //     colors::WHITE,
                // );

                // let v = get_orbital_velocity_compensated(ball, &static_bodies[0], dt);
            }

            for body in &self.static_bodies {
                body.draw();
            }

            // quad_tree.debug_draw();

            // Draw trace objects
            // for trace in traces {
            //     draw_circle(trace.x, trace.y, 1., colors::BLUE);
            // }

            match self.ball_under {
                Some(entry) => {
                    let b = self.balls[entry];
                    draw_circle_lines(b.position.x, b.position.y, b.radius + 3., 2., colors::GOLD);
                }
                _ => {}
            }
        }

        set_default_camera();
    }
}
