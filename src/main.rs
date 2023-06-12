use bevy::{
    math::{vec2, Vec3Swizzles},
    window::CursorGrabMode,
};

use anyhow::Result;
use bevy::{input::mouse::MouseMotion, math::vec3, prelude::*};
use rand::{rngs::ThreadRng, thread_rng, Rng};
const SCENE_LENGTH: usize = 30;

mod skybox;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(skybox::SkyBoxPlugin {})
        .add_startup_system(setup)
        .add_system(physics)
        .add_system(keyboard_input)
        .add_system(cursor_grab_system)
        .add_system(mouse_motion)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let plane = meshes.add(shape::Plane::from_size(SCENE_LENGTH as f32).into());
    // plane
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: materials.add(Color::rgb(0.2, 0.2, 0.5).into()),
        transform: Transform::from_xyz(SCENE_LENGTH as f32 * -0.25, 0.0, 0.0)
            .with_scale(vec3(0.5, 1.0, 1.0)),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: plane.clone(),
        material: materials.add(Color::rgb(0.5, 0.2, 0.2).into()),
        transform: Transform::from_xyz(SCENE_LENGTH as f32 * 0.25, 0.0, 0.0)
            .with_scale(vec3(0.5, 1.0, 1.0)),
        ..default()
    });
    let camera_id = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.5, 1.0).looking_at(-Vec3::Z, Vec3::Y),
            ..default()
        })
        .insert(Gimble::default())
        .id();
    // player
    let player_children = [
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.1,
                    depth: 0.3,
                    ..default()
                })),

                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 0.25, 0.0),
                ..default()
            })
            .id(),
        camera_id,
    ];
    let player = commands
        .spawn((
            Physics::default(),
            TransformBundle {
                local: Transform::from_xyz(0.0, 10., 0.0),
                ..default()
            },
            VisibilityBundle::default(),
        ))
        .insert_children(0, &player_children)
        .id();

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::Rgba {
                red: 1.0,
                green: 0.9,
                blue: 0.6,
                alpha: 1.0,
            },
            illuminance: 100_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: {
            let mut v = Transform::default();
            v.look_at(-vec3(0.5, 1., 0.5), Vec3::Y);
            v
        },
        ..default()
    });
    // fill light
    commands.insert_resource(AmbientLight {
        color: Color::Rgba {
            red: 0.6,
            green: 0.8,
            blue: 1.0,
            alpha: 1.0,
        },
        brightness: 3.0,
    });

    commands.insert_resource(MainPlayer {
        id: player,
        camera_id,
    });
    // camera
    let mut rng = thread_rng();
    let data = fun_name(&mut rng);
    // let nearest = gen_nearest(data);
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    for x in 0..SCENE_LENGTH {
        for y in 0..SCENE_LENGTH {
            let pos = (vec2(x as f32, y as f32) / SCENE_LENGTH as f32 - 0.5) * SCENE_LENGTH as f32;
            let value = data[x][y];
            if value == 1 {
                commands.spawn(PbrBundle {
                    mesh: mesh.clone(),

                    material: material.clone(),
                    transform: Transform::from_xyz(pos.x + 0.5, 0.5, pos.y + 0.5),
                    ..default()
                });
            };
        }
    }

    commands.insert_resource(SceneData { blocks: data })
}

fn gen_nearest(
    data: [[u8; SCENE_LENGTH]; SCENE_LENGTH],
) -> [[(usize, usize); SCENE_LENGTH]; SCENE_LENGTH] {
    let coords = (0..SCENE_LENGTH).flat_map(|x| (0..SCENE_LENGTH).map(move |y| (x, y)));
    let mut result = [[(0, 0); SCENE_LENGTH]; SCENE_LENGTH];
    for (x, y) in coords.clone() {
        let nearest = coords
            .clone()
            .filter(|(x2, y2)| data[*x2][*y2] == 1)
            .map(|(x2, y2)| {
                (
                    (x2 as isize - x as isize).pow(2) + (y2 as isize - y as isize).pow(2),
                    (x2, y2),
                )
            })
            .max_by_key(|(dist_sq, _)| *dist_sq)
            .unwrap()
            .1;
        result[x][y] = nearest;
    }
    result
}

fn fun_name(rng: &mut ThreadRng) -> [[u8; SCENE_LENGTH]; SCENE_LENGTH] {
    let mut data = [[0; SCENE_LENGTH]; SCENE_LENGTH];
    for tile in data.iter_mut().map(|slice| slice.iter_mut()).flatten() {
        *tile = rng.gen_range(0..5);
    }
    data
}

#[derive(Resource, Reflect, Debug)]
struct SceneData<const I: usize> {
    blocks: [[u8; I]; I],
}

#[derive(Reflect, Debug, Default, Component)]
struct Physics {
    velocity: Vec3,
    on_ground: bool,
}

#[derive(Resource, Reflect, Debug)]
struct MainPlayer {
    id: Entity,
    camera_id: Entity,
}

#[derive(Reflect, Debug, Default, Component)]
struct Gimble {
    theta: f32,
}

fn physics(
    mut players: Query<(&mut Transform, &mut Physics)>,
    time: Res<Time>,
    data: Res<SceneData<SCENE_LENGTH>>,
) {
    let delta = time.delta_seconds();
    for (mut transform, mut physics) in players.iter_mut() {
        do_scene_colisions(&mut transform, &mut physics, &data);
        physics.velocity.y -= delta * 9.81;
        transform.translation += physics.velocity * delta;
        if transform.translation.y <= 0.0 {
            physics.velocity.y = 0.0;
            physics.velocity.x *= 0.7;
            physics.velocity.z *= 0.7;
            transform.translation.y = 0.0;
            physics.on_ground = true;
        }
    }
}

fn do_scene_colisions(
    transform: &mut Transform,
    physics: &mut Physics,
    data: &Res<SceneData<SCENE_LENGTH>>,
) {
    let grid_coord = ((Vec3Swizzles::xz(transform.translation) / SCENE_LENGTH as f32 + 0.5)
        * SCENE_LENGTH as f32);
    for (x, y) in (-1..1).flat_map(|x| (-1..1).map(move |y| (x, y))) {
        let cube_coord = vec2(x as f32, y as f32) + grid_coord;
        let cube_position = vec2(cube_coord.x as f32, cube_coord.y as f32) - 0.5;
        if (transform.translation.x - cube_position.x).abs() - 0.6 < 0.0
            && (transform.translation.y - cube_position.y).abs() - 0.6 < 0.0
        {
            physics.velocity = physics.velocity
                * -physics
                    .velocity
                    .dot(cube_position.extend(0.0).xzy() - transform.translation);
        }
    }
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    main_player: Res<MainPlayer>,
    mut player: Query<(&mut Physics, &Transform)>,
    time: Res<Time>,
) {
    let _ = || -> Result<()> {
        let (mut physics, transform) = player.get_mut(main_player.id)?;
        let mut vel = Vec3::ZERO;
        let delta = time.delta_seconds();
        let speed = if physics.on_ground {
            if keys.any_pressed([KeyCode::LShift, KeyCode::RShift]) {
                2.0
            } else {
                1.0
            }
        } else {
            0.5
        } * 50.0;
        if keys.pressed(KeyCode::W) {
            vel += transform.forward();
        }
        if keys.pressed(KeyCode::A) {
            vel += transform.left();
        }
        if keys.pressed(KeyCode::S) {
            vel += transform.back();
        }
        if keys.pressed(KeyCode::D) {
            vel += transform.right();
        }
        physics.velocity += vel * delta * speed;

        if keys.just_pressed(KeyCode::Space) {
            if physics.on_ground {
                physics.velocity.y += 3.0; // Space was pressed
                physics.on_ground = false;
            }
        }
        if keys.any_just_pressed([KeyCode::Delete, KeyCode::Back]) {
            // Either delete or backspace was just pressed
        }
        if keys.just_released(KeyCode::LControl) {
            // Left Ctrl was released
        }
        Ok(())
    }();
}

fn mouse_motion(
    mut motion_evr: EventReader<MouseMotion>,
    player: Res<MainPlayer>,
    mut transform: Query<&mut Transform>,
    mut gimble: Query<&mut Gimble>,
) {
    let _ = || -> Result<()> {
        let [mut player_transform, mut camera_transform] =
            transform.get_many_mut([player.id, player.camera_id])?;
        let mut gimble = gimble.get_mut(player.camera_id)?;
        for ev in motion_evr.iter() {
            // println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
            const SENCITIVITY: f32 = 0.01;
            player_transform.rotate_y(ev.delta.x * -SENCITIVITY);
            let view_lock = 1.5707963268;
            gimble.theta = (gimble.theta + ev.delta.y * -SENCITIVITY)
                .min(view_lock)
                .max(-view_lock);
            camera_transform.rotation = Quat::from_rotation_x(gimble.theta);
        }

        Ok(())
    }();
}

fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.get_single_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.cursor.grab_mode = CursorGrabMode::Locked;
        window.cursor.visible = false;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
    }
}
