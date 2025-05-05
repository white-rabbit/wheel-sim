use bevy::prelude::*;
use rand::prelude::*;
pub mod components;
pub mod entity;

pub use components::*;
pub use entity::*;

#[derive(Debug, Default)]
pub struct WheelPlugin;

const TIME_SCALE: f32 = 3.0;
const EPSILON: f32 = 0.0001;

impl Plugin for WheelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                // wheel
                keyboard_control,
                simulate_wheel,
                // sparks
                simulate_strike_sparks,
                simulate_sparks_motion,
                dispose_expired_sparks,
                // motion
                sync_transforms,
            )
                .chain(),
        );
    }
}

// -- ground function and collision --

pub fn ground_func(x: f32) -> f32 {
    100.0 * f32::sin(x / 100.0) - 200.0 + 0.0001 * x * x + 0.1 * x
}

fn ground_dist(x: f32, x_wheel: f32, y_wheel: f32) -> f32 {
    let y = ground_func(x);
    let x_diff = x_wheel - x;
    let y_diff = y_wheel - y;
    x_diff * x_diff + y_diff * y_diff
}

fn min_distance(x: f32, x_wheel: f32, y_wheel: f32) -> f32 {
    let mut dx = 1.0;

    if ground_dist(x, x_wheel, y_wheel) < ground_dist(x + dx, x_wheel, y_wheel) {
        dx = -dx;
    }

    let mut x_cur = x;

    while f32::abs(dx) > EPSILON {
        if ground_dist(x_cur, x_wheel, y_wheel) > ground_dist(x_cur + dx, x_wheel, y_wheel) {
            x_cur += dx;
        } else {
            dx *= 0.5;
        }
    }

    x_cur
}

fn get_intersection(x: f32, y: f32, radius: f32) -> Result<Vec2, ()> {
    let left_min = min_distance(x - radius, x, y);
    let right_min = min_distance(x + radius, x, y);

    let left_dist = ground_dist(left_min, x, y);
    let right_dist = ground_dist(right_min, x, y);

    let (best_dist, best_x) = if left_dist < right_dist {
        (left_dist, left_min)
    } else {
        (right_dist, right_min)
    };

    if best_dist <= radius * radius {
        Ok(Vec2::new(best_x, ground_func(best_x)))
    } else {
        Err(())
    }
}

// -- wheel --

pub fn simulate_wheel(
    fixed_time: Res<Time<Fixed>>,
    wheel: Single<(
        &mut Pos,
        &mut Vel,
        &mut Rot,
        &mut RotVel,
        &Radius,
        &Mass,
        &mut ContactState,
    )>,
) {
    let dt = fixed_time.delta_secs() * TIME_SCALE;
    let (mut pos, mut vel, mut rot, mut rot_vel, radius, mass, mut state) = wheel.into_inner();

    let mut force = Vec2::new(0.0, 0.0);
    let mut torque = 0.0;

    let inertia_moment = mass.0 * radius.0 * radius.0 * 0.5;

    // gravity
    let gravity = Vec2::new(0.0, -mass.0 * 9.81);

    let contact = get_intersection(pos.0.x, pos.0.y, radius.0);

    if let Ok(contact) = contact {
        let distance = (contact - pos.0).length();
        let penetration = distance - radius.0;
        let normal: Vec2 = (contact - pos.0).normalize();
        let tangent = Vec2::new(normal.y, -normal.x);

        // fix position and velocity (this is incorrect)
        pos.0 += normal * penetration;
        let normal_vel = vel.0.dot(-normal);
        vel.0 += normal_vel * normal * 1.01;

        // compute forces
        let support_force_value = gravity.dot(-normal);
        let support_force = normal * support_force_value;

        let mut tangential_force = gravity - support_force;

        // friction
        let contact_vel = rot_vel.0 * radius.0 * tangent;
        let sliding_vel = vel.0 - contact_vel;

        let coeff = 0.2;
        let friction = sliding_vel * support_force_value * coeff;

        force += 2.0 / 3.0 * tangential_force;

        tangential_force -= friction;

        let tangential_force_value = tangential_force.dot(tangent);
        torque = 1.0 / 3.0 * tangential_force_value * radius.0;

        force += friction;

        // recrod contact data
        state.has_contact = true;
        state.contact_pos = contact;
        state.sliding_vel = sliding_vel;
    } else {
        force += gravity;

        // clear contact data
        state.has_contact = false;
    }

    pos.0 += vel.0 * dt;
    vel.0 += force / mass.0 * dt;
    rot.0 += rot_vel.0 * dt;
    rot_vel.0 += torque / inertia_moment * dt;
}

// -- keyboard control --

pub fn keyboard_control(
    keyboard: Res<ButtonInput<KeyCode>>,
    wheel: Single<(&mut Vel, &mut RotVel)>,
) {
    let dx = 1.0;
    let dr = 0.01;

    let (mut pos, mut rot) = wheel.into_inner();

    if keyboard.pressed(KeyCode::KeyA) {
        pos.0.x -= dx;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        pos.0.x += dx;
    }

    if keyboard.pressed(KeyCode::KeyW) {
        pos.0.y += dx;
    }

    if keyboard.pressed(KeyCode::KeyS) {
        pos.0.y -= dx;
    }

    if keyboard.pressed(KeyCode::KeyQ) {
        rot.0 += dr;
    }

    if keyboard.pressed(KeyCode::KeyE) {
        rot.0 -= dr;
    }
}

// -- sparks --

pub fn simulate_strike_sparks(mut commands: Commands, wheel: Single<&ContactState>) {
    let state = wheel.into_inner();

    if state.has_contact {
        let sliding_vel_length = state.sliding_vel.length();
        let sparks_count = 5 * (sliding_vel_length / 30.0) as u32;

        for _ in 0..sparks_count {
            let random_unit_vec = Vec2::new(
                thread_rng().gen_range(-1.0..1.0),
                thread_rng().gen_range(-1.0..1.0),
            )
            .normalize();
            let vel = 6.0 * state.sliding_vel + random_unit_vec * sliding_vel_length;
            let spark_vel_length = vel.length();
            let lifetime = f32::clamp(0.0005 * spark_vel_length * spark_vel_length, 10.0, 100.0);
            commands.spawn((Particle(lifetime), Pos(state.contact_pos), Vel(vel)));
        }
    }
}

pub fn simulate_sparks_motion(
    fixed_time: Res<Time<Fixed>>,
    mut particles: Query<(&mut Pos, &mut Vel, &mut Particle)>,
) {
    let dt = fixed_time.delta_secs() * TIME_SCALE;
    for particle in particles.iter_mut() {
        let (mut pos, mut vel, mut particle) = particle;
        particle.0 -= 1.0;
        pos.0 += vel.0 * dt;
        vel.0.y += -9.81 * dt;
    }
}

// TODO: use events instead
fn dispose_expired_sparks(mut commands: Commands, query: Query<(Entity, &Particle)>) {
    for (eid, particle) in &query {
        if particle.0 < 0.0 {
            commands.entity(eid).despawn();
        }
    }
}

// -- common --

pub fn sync_transforms(
    mut query: Query<(&mut bevy::transform::components::Transform, &Pos, &Rot)>,
) {
    for (mut transform, pos, rot) in query.iter_mut() {
        transform.translation = pos.0.extend(0.);
        transform.rotation = Quat::from_rotation_z(rot.0);
    }
}
