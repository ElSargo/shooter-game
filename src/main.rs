use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    math::{vec2, Vec3Swizzles},
    render::render_resource::Extent3d,
    window::CursorGrabMode,
};

use anyhow::Result;
use bevy::{input::mouse::MouseMotion, math::vec3, prelude::*};

use main_material::MainMaterial;
use rand::{rngs::ThreadRng, thread_rng, Rng};
const SCENE_LENGTH: usize = 30;

mod main_material;
mod skybox;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        // .add_plugins(DefaultPlugins)
        .add_plugin(skybox::SkyboxPlugin)
        .add_plugin(main_material::MainMaterialPlugin)
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
    mut materials: ResMut<Assets<MainMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let plane = meshes.add(shape::Plane::from_size(SCENE_LENGTH as f32).into());

    let mut rng = thread_rng();
    let data: [[f32; SCENE_LENGTH]; SCENE_LENGTH] = fun_name(&mut rng);
    let box_texture = images.add(Image::new(
        Extent3d {
            width: SCENE_LENGTH as u32,
            height: SCENE_LENGTH as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        {
            let coords = 0..SCENE_LENGTH;
            coords
                .clone()
                .flat_map(|x| coords.clone().map(move |y| (x, y)))
                .flat_map(|(x, y)| data[y][x].to_ne_bytes())
                .collect()
        },
        bevy::render::render_resource::TextureFormat::R32Float,
    ));
    let white_material = materials.add(MainMaterial {
        color: Color::rgb(1.0, 1.0, 1.0),
        boxes: Some(box_texture.clone()),
        ..default()
    });

    let blue_material = materials.add(MainMaterial {
        color: Color::Rgba {
            red: 0.4,
            green: 0.4,
            blue: 1.0,
            alpha: 1.0,
        },
        boxes: Some(box_texture.clone()),
        ..default()
    });

    let red_material = materials.add(MainMaterial {
        color: Color::Rgba {
            red: 1.0,
            green: 0.4,
            blue: 0.4,
            alpha: 1.0,
        },
        boxes: Some(box_texture.clone()),

        ..default()
    });
    // plane
    commands.spawn(MaterialMeshBundle {
        mesh: plane.clone(),
        material: blue_material.clone(),
        transform: Transform::from_xyz(SCENE_LENGTH as f32 * -0.25, 0.0, 0.0)
            .with_scale(vec3(0.5, 1.0, 1.0)),
        ..default()
    });
    commands.spawn(MaterialMeshBundle {
        mesh: plane.clone(),
        material: red_material.clone(),
        transform: Transform::from_xyz(SCENE_LENGTH as f32 * 0.25, 0.0, 0.0)
            .with_scale(vec3(0.5, 1.0, 1.0)),
        ..default()
    });
    let camera_id = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::AcesFitted,
            transform: Transform::from_xyz(0.0, 0.0, 0.5).looking_at(-Vec3::Z, Vec3::Y),
            ..default()
        })
        .id();
    // player
    let gimble_id = commands
        .spawn((
            Gimble::default(),
            TransformBundle {
                local: Transform::from_xyz(0.0, 0.4, 0.0),
                ..default()
            },
            VisibilityBundle::default(),
        ))
        .insert_children(0, &[camera_id])
        .id();
    let player_children = [
        commands
            .spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: -0.125,
                    max_x: 0.125,
                    min_y: -0.125,
                    max_y: 0.125,
                    min_z: -0.125,
                    max_z: 0.125,
                })),

                material: white_material.clone(),

                transform: Transform::from_xyz(0.0, 0.125, 0.0),
                ..default()
            })
            .id(),
        gimble_id,
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

    commands.insert_resource(MainPlayer {
        id: player,
        gimble_id,
        camera_id,
    });
    // camera
    // let nearest = gen_nearest(data);
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.001 }));
    for x in 0..SCENE_LENGTH {
        for y in 0..SCENE_LENGTH {
            let pos = (vec2(x as f32, y as f32) / SCENE_LENGTH as f32 - 0.5) * SCENE_LENGTH as f32;
            let value = data[x][y];
            if value > 0.6 {
                let box_height = value * 0.5;
                commands.spawn((
                    Cube {
                        x: x as usize,
                        y: y as usize,
                    },
                    MaterialMeshBundle {
                        mesh: mesh.clone(),

                        material: white_material.clone(),
                        transform: Transform::from_xyz(pos.x + 0.5, box_height, pos.y + 0.5)
                            .with_scale(vec3(1.0, value, 1.0)),
                        ..default()
                    },
                ));
            };
        }
    }

    commands.insert_resource(SceneData { blocks: data });
}

#[derive(Debug, Component, Reflect, Clone, Copy)]
struct Cube {
    x: usize,
    y: usize,
}
// fn gen_nearest(
//     data: [[f32; SCENE_LENGTH]; SCENE_LENGTH],
// ) -> [[(usize, usize); SCENE_LENGTH]; SCENE_LENGTH] {
//     let coords = (0..SCENE_LENGTH).flat_map(|x| (0..SCENE_LENGTH).map(move |y| (x, y)));
//     let mut result = [[(0, 0); SCENE_LENGTH]; SCENE_LENGTH];
//     for (x, y) in coords.clone() {
//         let nearest = coords
//             .clone()
//             .filter(|(x2, y2)| data[*x2][*y2] == 1)
//             .map(|(x2, y2)| {
//                 (
//                     (x2 as isize - x as isize).pow(2) + (y2 as isize - y as isize).pow(2),
//                     (x2, y2),
//                 )
//             })
//             .max_by_key(|(dist_sq, _)| *dist_sq)
//             .unwrap()
//             .1;
//         result[x][y] = nearest;
//     }
//     result
// }

fn fun_name(rng: &mut ThreadRng) -> [[f32; SCENE_LENGTH]; SCENE_LENGTH] {
    let mut data = [[0.0; SCENE_LENGTH]; SCENE_LENGTH];
    for tile in data.iter_mut().map(|slice| slice.iter_mut()).flatten() {
        *tile = rng.gen_range(0.0..1.0);
    }
    data
}

#[derive(Resource, Debug)]
struct SceneData<const I: usize> {
    blocks: [[f32; I]; I],
}

#[derive(Reflect, Debug, Default, Component)]
struct Physics {
    velocity: Vec3,
    on_ground: bool,
}

#[derive(Resource, Reflect, Debug)]
struct MainPlayer {
    id: Entity,
    gimble_id: Entity,
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
    let grid_coord =
        Vec3Swizzles::xz(transform.translation) / SCENE_LENGTH as f32 + 0.5 * SCENE_LENGTH as f32;
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
        let [mut player_transform, mut gimble_transform] =
            transform.get_many_mut([player.id, player.gimble_id])?;
        let mut gimble = gimble.get_mut(player.gimble_id)?;
        for ev in motion_evr.iter() {
            // println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
            const SENCITIVITY: f32 = 0.01;
            player_transform.rotate_y(ev.delta.x * -SENCITIVITY);
            let view_lock = 1.5707963268;
            gimble.theta = (gimble.theta + ev.delta.y * -SENCITIVITY)
                .min(view_lock)
                .max(-view_lock);
            gimble_transform.rotation = Quat::from_rotation_x(gimble.theta);
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
