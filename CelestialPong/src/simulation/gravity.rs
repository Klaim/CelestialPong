use macroquad::{
    color::{colors, Color},
    math::Vec2,
    shapes::draw_line,
};

use crate::simulation::ball::Ball;
use crate::SIMULATION_DT;

const GRAVITY: f32 = 1.;

pub fn get_gravity_force(ball: &Ball, body: &Ball) -> Vec2 {
    let delta = body.position - ball.position;
    return delta.normalize() * (body.mass) / delta.length().powf(2.) * GRAVITY;
}

#[allow(dead_code)]
pub fn get_gravity_radius_over_threshold(mass: f32, threshold: f32) -> f32 {
    return (mass * GRAVITY / threshold).sqrt();
}

pub fn get_orbital_velocity(b1: &Ball, body: &Ball) -> Vec2 {
    let delta = body.position - b1.position;
    let orbit_radius = delta.length();
    let speed = (GRAVITY * (body.mass) / orbit_radius).sqrt();
    return Vec2::from((delta.y, -delta.x)).normalize() * speed;
}

#[allow(dead_code)]
pub fn get_orbital_period(ball: &Ball, body: &Ball) -> f32 {
    let delta = body.position - ball.position;
    let orbit_radius = delta.length();
    let a = orbit_radius * orbit_radius * orbit_radius;
    let d = GRAVITY * body.mass;
    let sqrt = (a / d).sqrt();
    return sqrt * std::f32::consts::TAU / SIMULATION_DT;
}

#[allow(dead_code)]
pub fn get_orbital_velocity_compensated(b1: &Ball, body: &Ball, dt: f32) -> Vec2 {
    let base_vel = get_orbital_velocity(b1, body);
    let stepped_angle = get_orbital_period(b1, body) / dt;
    let expected =
        Vec2::from_angle(stepped_angle).rotate(b1.position - body.position) + body.position;
    let expected = expected - b1.position;

    draw_arrow(b1.position, base_vel / 30., 1., colors::GOLD);
    draw_arrow(b1.position, expected / 1., 1., colors::SKYBLUE);

    return expected;
}

pub fn damping(pos: Vec2, target: Vec2, dt: f32, elasticity: f32) -> Vec2 {
    return (target - pos) / elasticity * dt;
}

pub fn draw_arrow(origin: Vec2, vec: Vec2, thickness: f32, color: Color) {
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
