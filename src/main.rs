use bevy::{
    core_pipeline::{
        bloom::{Bloom, BloomCompositeMode},
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use bevy_fps_counter::FpsCounterPlugin;

use wheel_phys::components::{Mass, Particle, Pos, RotVel, Vel};
use wheel_phys::entity::WheelBundle;
use wheel_phys::WheelPlugin;

fn key_indicator(commands: &mut Commands, symbol: &str, color: &Color, txt_x: f32, txt_y: f32) {
    let padding = 15.0;
    commands.spawn((
        Text::new(symbol),
        TextColor(*color),
        TextFont {
            font_size: 35.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(txt_y + padding),
            right: Val::Px(txt_x + padding),
            ..default()
        },
    ));
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        Camera {
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 3. Enable bloom for the camera
    ));

    let wasd_color = TextColor(Color::srgb(0.4, 0.4, 1.0));
    let qe_color = TextColor(Color::srgb(1.0, 0.3, 0.3));

    let txt_x = 50.0;
    let txt_y = 50.0;

    // QWE
    key_indicator(&mut commands, "Q", &qe_color, txt_x * 2.0, txt_y);
    key_indicator(&mut commands, "W", &wasd_color, txt_x, txt_y);
    key_indicator(&mut commands, "E", &qe_color, 0.0, txt_y);

    // ASD
    key_indicator(&mut commands, "A", &wasd_color, txt_x * 2.0, 0.0);
    key_indicator(&mut commands, "S", &wasd_color, txt_x, 0.0);
    key_indicator(&mut commands, "D", &wasd_color, 0.0, 0.0);

    let image_handle = asset_server.load("rusty_wheel.png");
    let mut sprite = Sprite::from_image(image_handle);

    sprite.custom_size = Some(Vec2::new(100.0, 100.0));

    // wheel
    commands
        .spawn(WheelBundle::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)))
        .insert((Transform::from_xyz(0.0, 0.0, 0.0), sprite));
}

fn draw_curve(
    mut gizmos: Gizmos,
    windows: Query<&mut Window>,
    wheel_query: Query<&Pos, With<Mass>>,
) {
    let window = windows.single().expect("No window!");
    let width = window.resolution.width();

    let mut points = Vec::new();
    let pos = wheel_query.single().expect("No wheel!");

    let left_border = -width * 0.5 + pos.0.x;
    let right_border = width * 0.5 + pos.0.x;

    for i in 0..501 {
        let t = i as f32 / 500.0;
        let x = left_border + t * (right_border - left_border);
        let y = wheel_phys::ground_func(x);
        points.push(Vec3::new(x, y, 0.0));
    }

    gizmos.linestrip(points.clone(), Color::srgb(1.0, 0.7, 0.6));
}

fn draw_sparks(mut gizmos: Gizmos, particles: Query<(&Pos, &Particle)>) {
    for (pos, particle) in &particles {
        let x = pos.0.x;
        let y = pos.0.y;
        let glow = f32::max(particle.0 / 30.0, 0.7);

        gizmos
            .circle_2d(
                Isometry2d::new(Vec2::new(x, y), Rot2::radians(0.0)),
                1.0,
                Color::srgb(1.3 * glow, 0.8 * glow, 0.5 * glow),
            )
            .resolution(64);
    }
}

fn change_wheel_color(wheel: Single<(&mut Sprite, &Vel)>) {
    let (mut wheel, vel) = wheel.into_inner();
    let factor = 2.0 + vel.0.length() * 0.01;
    wheel.color = Color::srgb(0.95 * factor, 0.6 * factor, 0.4 * factor);
}

fn highlight_keys(keyboard: Res<ButtonInput<KeyCode>>, mut keys: Query<(&Text, &mut TextColor)>) {
    for (text, mut color) in keys.iter_mut() {
        let key_text = text.as_str();
        let key_pressed = match key_text {
            "Q" => keyboard.pressed(KeyCode::KeyQ),
            "W" => keyboard.pressed(KeyCode::KeyW),
            "E" => keyboard.pressed(KeyCode::KeyE),
            "A" => keyboard.pressed(KeyCode::KeyA),
            "S" => keyboard.pressed(KeyCode::KeyS),
            "D" => keyboard.pressed(KeyCode::KeyD),
            _ => false,
        };

        // Get the base color depending on whether it's a WASD key or QE key
        let is_wasd = matches!(key_text, "W" | "A" | "S" | "D");
        let base_color = if is_wasd {
            Vec3::new(0.4, 0.4, 1.0)
        } else {
            Vec3::new(1.0, 0.3, 0.3)
        };

        let factor = 1.7;

        // Increase brightness if key is pressed
        if key_pressed {
            color.0 = Color::srgb(
                base_color.x * factor,
                base_color.y * factor,
                base_color.z * factor,
            );
        } else {
            color.0 = Color::srgb(base_color.x, base_color.y, base_color.z);
        }
    }
}

fn follow_wheel(
    wheel_query: Query<&Pos, Without<Particle>>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(pos) = wheel_query.single() {
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            // Follow the wheel's X position, but keep Y stable
            camera_transform.translation.x = pos.0.x;
            camera_transform.translation.y = pos.0.y;
        }
    }
}

fn update_bloom_settings(
    camera: Single<(Entity, Option<&mut Bloom>), With<Camera>>,
    wheel: Single<&RotVel>,
) {
    let bloom = camera.into_inner();
    let rot_vel = wheel.into_inner();

    match bloom {
        (_entity, Some(mut bloom)) => {
            bloom.intensity = 0.11 + 0.14 * rot_vel.0.abs();
            bloom.low_frequency_boost = 0.2;
            bloom.low_frequency_boost_curvature = 0.4;
            bloom.high_pass_frequency = 0.1 + rot_vel.0.abs();
            bloom.composite_mode = BloomCompositeMode::Additive;
            bloom.prefilter.threshold = -0.7;
            bloom.prefilter.threshold_softness = 0.7;
        }
        (_entity, None) => (),
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.03, 0.03)))
        .add_plugins((DefaultPlugins, WheelPlugin, FpsCounterPlugin))
        .add_systems(Startup, startup)
        .add_systems(
            Update,
            (
                draw_curve,
                draw_sparks,
                change_wheel_color,
                update_bloom_settings,
                highlight_keys,
                follow_wheel,
            ),
        )
        .run();
}
