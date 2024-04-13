// based on https://github.com/Markek1/Collision-Simulator
// other usefull link https://arrowinmyknee.com/2021/03/15/some-math-about-capsule-collision/

mod ball;
mod capsule;
mod quad_tree;

use macroquad::{
    color::{self, colors},
    prelude::*,
    rand::{srand, RandomRange},
    window,
};

use macroquad::rand;

use crate::ball::*;
use crate::quad_tree::*;

const WINDOW_SIZE: [f32; 2] = [1000., 1000.];

const SCENE_SIZE: [f32; 2] = [800., 1024.];

const NB_BALLS: usize = 220;
const RADII: f32 = 3.;
const BALL_MASS: f32 = 40.;

const GRAVITY: f32 = 10.;

const BODY_MASS: f32 = 5000000.;
const BODY_BOUNCYNESS: f32 = 0.9;

const ORBIT_TRAP: f32 = 10.0;
const ORBIT_TRAP_SIZE: f32 = 4.;

const MIN_START_ORBIT: f32 = 160.;
const MAX_START_ORBIT: f32 = 301.;

const FPS_FRAMES: usize = 100;
const TRACE_SIZE: usize = 1000;

const SIMULATION_DT: f32 = 1. / 240.;

fn draw_arrow(origin: Vec2, vec: Vec2, thickness: f32, color: Color) {
    let dest = vec + origin;
    let n1 = (Vec2::from((vec.y, -vec.x)) - vec) / 4.;
    let n2 = (Vec2::from((-vec.y, vec.x)) - vec) / 4.;
    draw_line(origin.x, origin.y, dest.x, dest.y, thickness, color);

    draw_line(
        dest.x,
        dest.y,
        dest.x + n1.x,
        dest.y + n1.y,
        thickness,
        color,
    );
    draw_line(
        dest.x,
        dest.y,
        dest.x + n2.x,
        dest.y + n2.y,
        thickness,
        color,
    );
}

fn damping(pos: Vec2, target: Vec2, dt: f32, elasticity: f32) -> Vec2 {
    return (target - pos) / elasticity * dt;
}

fn get_gravity_force(ball: &Ball, body: &Ball) -> Vec2 {
    let delta = body.position - ball.position;
    return delta.normalize() * (body.mass) / delta.length().powf(2.) * GRAVITY;
}

fn get_gravity_radius_over_threshold(mass: f32, threshold: f32) -> f32 {
    return (mass * GRAVITY / threshold).sqrt();
}

fn get_orbital_velocity(b1: &Ball, body: &Ball) -> Vec2 {
    let delta = body.position - b1.position;
    let orbit_radius = delta.length();
    let speed = (GRAVITY * (body.mass) / orbit_radius).sqrt();
    return Vec2::from((delta.y, -delta.x)).normalize() * speed;
}

fn get_orbital_period(ball: &Ball, body: &Ball) -> f32 {
    let delta = body.position - ball.position;
    let orbit_radius = delta.length();
    let a = orbit_radius * orbit_radius * orbit_radius;
    let d = GRAVITY * body.mass;
    let sqrt = (a / d).sqrt();
    return sqrt;
}

fn get_orbital_velocity_compensated(b1: &Ball, body: &Ball, dt: f32) -> Vec2 {
    let base_vel = get_orbital_velocity(b1, body);
    let next_position = b1.position + base_vel * dt;
    let stepped_angle = get_orbital_period(b1, body) / dt;
    println!("angle {}", stepped_angle);
    let expected =
        Vec2::from_angle(stepped_angle).rotate(b1.position - body.position) + body.position;
    let expected = expected - b1.position;

    draw_arrow(b1.position, base_vel / 30., 1., colors::GOLD);
    draw_arrow(b1.position, expected / 1., 1., colors::SKYBLUE);

    return expected;
}

fn random_orbital_pos(center: Vec2, min_radius: f32, max_radius: f32) -> Vec2 {
    let angle = RandomRange::gen_range(0., std::f32::consts::PI * 2.);
    let result = Vec2::from((angle.cos(), angle.sin()));
    let rad = RandomRange::gen_range(min_radius, max_radius);
    let result = center + result * rad;
    return result;
}

fn window_config() -> Conf {
    Conf {
        window_title: "Celestial pong".to_owned(),
        window_width: WINDOW_SIZE[0] as i32,
        window_height: WINDOW_SIZE[1] as i32,
        ..Default::default()
    }
}

fn reset_balls(balls: &mut Vec<Ball>, tree_area: quad_tree::Rect, static_bodies: &Vec<Ball>) {
    balls.clear();

    for _ in 0..NB_BALLS {
        let position =
            random_orbital_pos(static_bodies[0].position, MIN_START_ORBIT, MAX_START_ORBIT);

        let mut ball = Ball::new(
            position,
            Vec2::ZERO,
            RADII,
            BALL_MASS,
            Color {
                r: RandomRange::gen_range(0.25, 1.0),
                g: RandomRange::gen_range(0.25, 1.0),
                b: RandomRange::gen_range(0.25, 1.0),
                a: 1.,
            },
            tree_area,
        );

        // let ball_speed = Vec2::from((rng.gen::<f32>() * 20. - 10., rng.gen::<f32>() * 20. - 10.));
        let ball_speed = get_orbital_velocity(&ball, &static_bodies[0]);
        let _corrected_ball_speed =
            get_orbital_velocity_compensated(&ball, &static_bodies[0], SIMULATION_DT);

        ball.set_velocity(ball_speed, SIMULATION_DT);
        balls.push(ball);
    }
}

#[macroquad::main(window_config)]
async fn main() {
    let play_area_size = Vec2::new(window::screen_width(), window::screen_height());

    srand(1);
    let mut paused = true;
    let mut drawing_enabled = true;

    let mut balls = Vec::with_capacity(NB_BALLS);
    let mut static_bodies = Vec::new();

    let mut fps: [f32; FPS_FRAMES] = [0.; FPS_FRAMES];
    let mut fps_index: usize = 0;
    let mut frame_count = 0;

    let mut selected_ball: Option<usize> = None;

    let tree_area = quad_tree::Rect::new(0., 0., play_area_size.x * 4., play_area_size.x * 4.);

    let mut quad_tree = QuadTree::new(tree_area);

    let mut frame_per_frame: usize = 1;

    let mut collided_balls = Vec::with_capacity(NB_BALLS);
    let mut balls_marked_for_delete = Vec::with_capacity(NB_BALLS);

    static_bodies.push(Ball::new(
        Vec2::new(0., 0.),
        Vec2::ZERO,
        30.,
        BODY_MASS,
        color::WHITE,
        tree_area,
    ));

    let mut traces = [Vec2::ZERO; TRACE_SIZE];
    let mut trace_index = 0;

    let main_camera = Camera2D {
        zoom: Vec2::from((2. / WINDOW_SIZE[0], 2. / WINDOW_SIZE[1])),
        ..Default::default()
    };

    reset_balls(&mut balls, tree_area, &static_bodies);

    loop {
        if is_key_pressed(KeyCode::Escape) {
            return;
        }

        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::V) {
            drawing_enabled = !drawing_enabled;
        }

        if is_key_down(KeyCode::S) {
            for ball in &mut balls {
                ball.set_velocity(ball.velocity * 0.5, SIMULATION_DT);
            }
        }

        if is_key_down(KeyCode::R) {
            selected_ball = None;
            paused = true;
            frame_count = 0;
            srand(1);
            reset_balls(&mut balls, tree_area, &static_bodies);
        }

        if is_key_down(KeyCode::O) {
            for ball in &mut balls {
                ball.set_velocity(get_orbital_velocity(ball, &static_bodies[0]), SIMULATION_DT);
            }
        }

        if is_key_pressed(KeyCode::Up) {
            frame_per_frame = frame_per_frame + 1;
        }

        if is_key_pressed(KeyCode::Down) {
            frame_per_frame = (frame_per_frame - 1).max(1);
        }

        let dt = get_frame_time();
        fps[fps_index] = dt;
        fps_index = (fps_index + 1) % FPS_FRAMES;

        let dt = SIMULATION_DT;

        if !paused {
            for _ in 0..frame_per_frame {
                frame_count = frame_count + 1;

                quad_tree = QuadTree::new(tree_area);
                // Updating ball position
                collided_balls.clear();
                let nb_balls = balls.len();
                for index in 0..nb_balls {
                    let ball = balls.get(index).unwrap();
                    quad_tree.add(QuadTreeEntry::new(ball.position, index));
                    let mut local_force = Vec2::ZERO;

                    // Comuting gravity
                    if selected_ball == None || selected_ball.unwrap() != index {
                        for body in &static_bodies {
                            local_force = local_force + get_gravity_force(ball, body)
                        }
                    }

                    // Trapping ball in the nearest body
                    match static_bodies.iter().min_by(|body, other| {
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
                            if delta.length_squared() > ball.radius * ball.radius {
                                local_force = local_force + (delta / delta.length()) * ORBIT_TRAP;
                            }
                        }
                        _ => {}
                    }

                    let ball = balls.get_mut(index).unwrap();
                    // ball.update(dt, local_force);
                    ball.update_verlet(dt, local_force);

                    // Recode previous positions
                    traces[trace_index] = ball.position;
                    trace_index = (trace_index + 1) % traces.len();
                }

                // Colliding balls
                for index in 0..balls.len() {
                    // Has ball already collided this frame
                    if collided_balls.iter().any(|c| c == &index) {
                        continue;
                    }

                    let zone_check = balls[index].get_collision_area();
                    let mut near_balls = Vec::new();
                    quad_tree.query_entries(&zone_check, &mut near_balls);
                    for entry in near_balls {
                        if entry.payload == index
                            || collided_balls.iter().any(|c| c == &entry.payload)
                        {
                            continue;
                        }

                        let other_ball_index = entry.payload;

                        if balls[index].check_collision(&balls[other_ball_index]) {
                            if index > other_ball_index {
                                let (left, right) = balls.split_at_mut(index);
                                right[0].collide(&mut left[other_ball_index], dt);
                            } else {
                                let (left, right) = balls.split_at_mut(other_ball_index);
                                right[0].collide(&mut left[index], dt);
                            }

                            collided_balls.push(index);
                            collided_balls.push(other_ball_index);
                        }
                    }
                }

                // Bounce of static bodies
                for body in static_bodies.iter_mut() {
                    let query = body.get_collision_area();
                    let mut near_objects = Vec::new();
                    quad_tree.query_entries(&query, &mut near_objects);
                    for near in near_objects {
                        let ball = balls.get_mut(near.payload).unwrap();
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
                            if !balls_marked_for_delete.contains(&near.payload) {
                                balls_marked_for_delete.push(near.payload);
                            }
                        }
                    }
                }

                balls_marked_for_delete.sort_unstable();
                for index in balls_marked_for_delete.iter().rev() {
                    balls.remove(*index);
                    match selected_ball {
                        Some(selected) => {
                            if index == &selected {
                                selected_ball = None;
                            } else if index < &selected {
                                selected_ball = Some(selected - 1);
                            }
                        }
                        None => {}
                    }
                }

                balls_marked_for_delete.clear();
            }
        }

        let (spx, spy) = mouse_position();
        let mouse_pos = Vec2::new(spx, spy);
        let mouse_pos = main_camera.screen_to_world(mouse_pos);
        let dist_check = RADII * RADII * 10.;
        let mut near_balls = Vec::new();
        quad_tree.query_entries(
            &quad_tree::Rect::new(mouse_pos.x, mouse_pos.y, dist_check, dist_check),
            &mut near_balls,
        );

        let under = near_balls
            .into_iter()
            .find(|b| (balls[b.payload].position - mouse_pos).length_squared() < dist_check);

        if is_mouse_button_pressed(MouseButton::Left) {
            match under {
                Some(entry) => {
                    selected_ball = Some(entry.payload);
                }
                _ => {}
            }
        }

        if is_mouse_button_released(MouseButton::Left) {
            selected_ball = None;
        }

        match selected_ball {
            Some(ball_index) => {
                let ball = balls.get_mut(ball_index).unwrap();
                let force = damping(ball.position, mouse_pos, dt, 0.05 * dt);
                ball.set_velocity(force, dt);
            }
            _ => {}
        }

        if drawing_enabled {
            set_camera(&main_camera);

            for ball in &balls {
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

            for body in &static_bodies {
                body.draw();
            }

            // quad_tree.debug_draw();

            // Draw trace objects
            // for trace in traces {
            //     draw_circle(trace.x, trace.y, 1., colors::BLUE);
            // }

            match under {
                Some(entry) => {
                    let b = balls[entry.payload];
                    draw_circle_lines(b.position.x, b.position.y, b.radius + 3., 2., colors::GOLD);
                }
                _ => {}
            }
        }

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

            draw_text_ex(
                &format!("Frame count : {}", frame_count),
                32.,
                65.,
                TextParams {
                    font_size: 15,
                    ..Default::default()
                },
            );
        }

        next_frame().await
    }
}
