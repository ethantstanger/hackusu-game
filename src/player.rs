use bevy::prelude::*;
use {
    crate::{
        constants::{
            BOOST_ACCELERATION_SPEED, BULLET_SPEED, BULLET_VELOCITY_OFFSET, DRAG, MAX_SPEED,
            PASSIVE_ACCELERATION_SPEED, ROTATION_SPEED,
        },
        enemies::Enemy,
        jerry_cans::spawn_jerry_can,
    },
    bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    rand::{thread_rng, Rng},
    std::{f32::consts::TAU, time::Duration},
};

#[derive(Bundle)]
pub struct Player {
    sprite_bundle: SpriteBundle,
    velocity: Velocity,
    player_gun: PlayerStats,
}

#[derive(Component)]
pub struct PlayerStats {
    pub score: u32,
    shoot_timer: Timer,
    pub ammunition: u32,
}

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub fn setup_player(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    commands.spawn(Player {
        sprite_bundle: SpriteBundle {
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            texture: asset_server.load("graphics/player.png"),
            ..default()
        },
        velocity: Velocity(Vec2::ZERO),
        player_gun: PlayerStats {
            score: 0,
            shoot_timer: Timer::new(Duration::from_millis(5), TimerMode::Once),
            ammunition: 100,
        },
    });
}

pub fn control_player(
    time: Res<Time>,
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerStats, &mut Transform, &mut Velocity)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (mut player_stats, mut transform, mut velocity) = match query.get_single_mut() {
        Ok(value) => value,
        Err(_) => return,
    };

    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        transform.rotate_z(ROTATION_SPEED * time.delta_seconds());
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        transform.rotate_z(-ROTATION_SPEED * time.delta_seconds());
    }

    let axis_angle = transform.rotation.to_axis_angle();
    let current_rotation = axis_angle.0.z * axis_angle.1;

    player_stats.shoot_timer.tick(time.delta());

    if (input.pressed(KeyCode::KeyK)
        || input.pressed(KeyCode::Space)
        || input.pressed(KeyCode::KeyX)
        || input.pressed(KeyCode::ShiftRight))
        && player_stats.shoot_timer.finished()
        && player_stats.ammunition > 0
    {
        velocity.0 += Vec2::from_angle(current_rotation) * BOOST_ACCELERATION_SPEED;
        player_stats.ammunition -= 1;
        player_stats.shoot_timer.reset();
        spawn_bullets(
            10,
            transform.clone(),
            Some(current_rotation),
            &mut commands,
            &mut meshes,
            &mut materials,
        );
    }

    let velocity_speed = velocity.0.length();
    velocity.0 += Vec2::from_angle(current_rotation) * PASSIVE_ACCELERATION_SPEED;
    velocity.0 = velocity.0.normalize()
        * if velocity_speed > MAX_SPEED {
            MAX_SPEED
        } else {
            velocity_speed
        };
    velocity.0 *= DRAG;
}

#[derive(Component)]
pub struct Bullet {
    timer: Timer,
}

pub fn spawn_bullets(
    count: u32,
    spawn_transform: Transform,
    spawn_rotation: Option<f32>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    for _i in 0..count {
        let velocity =
            (Vec2::from_angle(spawn_rotation.unwrap_or(thread_rng().gen_range(-TAU..TAU)))
                * BULLET_SPEED
                * -1.0)
                + Vec2::from_angle(thread_rng().gen_range(-TAU..TAU)) * BULLET_VELOCITY_OFFSET;

        let color_int = thread_rng().gen_range(0..6);

        let color = if color_int <= 1 {
            Color::rgb(0.75, 0.1, 0.1)
        } else if color_int <= 2 {
            Color::rgb(0.86, 0.38, 0.1)
        } else if color_int <= 4 {
            Color::rgb(0.86, 0.63, 0.1)
        } else {
            Color::rgb(0.2, 0.2, 0.2)
        };

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle {
                    radius: thread_rng().gen_range(1.0..2.5),
                })),
                material: materials.add(color),
                transform: Transform {
                    translation: Vec3 {
                        z: 0.0,
                        ..spawn_transform.translation
                    },
                    rotation: Quat::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), velocity.to_angle()),
                    ..default()
                },
                ..default()
            },
            Velocity(velocity),
            Bullet {
                timer: Timer::new(
                    Duration::from_millis(thread_rng().gen_range(50..250)),
                    TimerMode::Once,
                ),
            },
        ));
    }
}

pub fn delete_bullets(
    time: Res<Time>,
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Bullet)>,
) {
    for (bullet, mut bullet_timer) in bullets.iter_mut() {
        bullet_timer.timer.tick(time.delta());

        if bullet_timer.timer.finished() {
            commands.entity(bullet).despawn();
        }
    }
}

pub fn kill_player(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), (With<PlayerStats>, Without<Enemy>)>,
    enemies: Query<&Transform, With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let (player_entity, player_transform) = match player_query.get_single() {
        Ok(value) => value,
        Err(_) => return,
    };

    for enemy in enemies.iter() {
        if enemy.translation.distance(Vec3 {
            x: player_transform.translation.x,
            y: player_transform.translation.y,
            z: enemy.translation.z,
        }) < 7.0
        {
            commands.entity(player_entity).despawn();
            spawn_bullets(
                45,
                player_transform.clone(),
                None,
                &mut commands,
                &mut meshes,
                &mut materials,
            );
            spawn_jerry_can(
                player_transform.translation,
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
            );
            break;
        }
    }
}

pub fn move_objects_with_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for object in query.iter_mut() {
        let (mut transform, velocity) = object;

        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();
    }
}
