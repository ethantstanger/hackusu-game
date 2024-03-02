use crate::constants::{
    BOOST_ACCELERATION_SPEED, DRAG, MAX_SPEED, PASSIVE_ACCELERATION_SPEED, ROTATION_SPEED,
};
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Velocity(Vec2);

pub fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("graphics/player.png"),
            ..default()
        },
        Velocity(Vec2::ZERO),
        Player,
    ));
}

pub fn move_player(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&Player, &mut Transform, &mut Velocity)>,
) {
    let (_player, mut transform, mut velocity) = query.single_mut();

    if input.pressed(KeyCode::KeyA) {
        transform.rotate_z(ROTATION_SPEED * time.delta_seconds());
    }
    if input.pressed(KeyCode::KeyD) {
        transform.rotate_z(-ROTATION_SPEED * time.delta_seconds());
    }

    let axis_angle = transform.rotation.to_axis_angle();
    let current_rotation = axis_angle.0.z * axis_angle.1;

    if input.pressed(KeyCode::KeyK) {
        let acceleration_vector = rotation_to_vector(current_rotation) * BOOST_ACCELERATION_SPEED;
        velocity.0 += acceleration_vector;
    }

    let velocity_speed = velocity.0.length();
    velocity.0 += rotation_to_vector(current_rotation) * PASSIVE_ACCELERATION_SPEED;
    velocity.0 = velocity.0.normalize()
        * if velocity_speed > MAX_SPEED {
            MAX_SPEED
        } else {
            velocity_speed
        };
    velocity.0 *= DRAG;

    transform.translation.x += velocity.0.x * time.delta_seconds();
    transform.translation.y += velocity.0.y * time.delta_seconds();
}

fn rotation_to_vector(rotation: f32) -> Vec2 {
    Vec2 {
        x: rotation.cos(),
        y: rotation.sin(),
    }
}
