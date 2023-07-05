use std::{f32::consts::TAU, ops::Mul};

use bevy::{
    math::{vec2, vec3, Vec3Swizzles},
    prelude::*,
};
use itertools::Itertools;
use rand::{thread_rng, Rng};

/*  ||=====================||  --             ||========||    /
    ||                     ||  |              ||        ||   /
    ||                     ||  | BODY_HEIGHT  ||        ||  /
--  =========================  --             ||        || /
|   |-----------------------|                 ||========||0
|<BODY_CLEAR, BODY_LENGTH^
|                                             |---------|
--                                             BODY_WIDTH
================================================================================

                                    --   /0\   --
                                    /   // \\   \
    ||=====================||   R1 /   //   \\   \
    ||                     ||     /   //     \\   \
    ||                     ||    /   //       \\   \   R2
    =========================   --   0         \\   \
                                   |-|          \\   \
                                LEG_WIDTH        \\   \
                                                  \\   \
                                                  [_]  --
================================================================================
*/

const BODY_CLEAR: f32 = 0.2;
// const BODY_HEIGHT: f32 = 0.2;
// const BODY_LENGTH: f32 = 0.3;
const BODY_WIDTH: f32 = 0.1;
const R1: f32 = 0.2;
const R2: f32 = 0.35;
const LEG_WIDTH: f32 = 0.02;
const NUM_LEGS: usize = 8;

pub struct CritterPlugin;
impl Plugin for CritterPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_critter)
            .add_system(coordinate_critter)
            .add_system(update_critter_mesh);
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Critter {
    pub velocity: Vec3,
    legs: [CritterLeg; NUM_LEGS],
    moving: bool,
    set_priorety: bool,
    just_moved: bool,
}

#[derive(Debug, Reflect)]
pub struct CritterLeg {
    local_body: Vec3,
    local_comfy_position: Vec3,
    comfy_distance: f32,
    global_knee: Vec3,
    global_body: Vec3,
    global_foot: Vec3,
    global_previous_target: Vec3,
    global_target: Vec3,
    t: f32,
    animation_speed: f32,
}

pub fn make_cirtter<T: Material>(
    commands: &mut Commands,
    material: Handle<T>,
    meshes: &mut ResMut<Assets<Mesh>>,
) -> Entity {
    let legs = (0..NUM_LEGS)
        .map(|x| (x as f32 / NUM_LEGS as f32 + 0.5 / NUM_LEGS as f32) * TAU)
        .map(|x| x.sin_cos())
        .map(|(x, y)| [vec2(x * 0.45, y * 0.25) * BODY_WIDTH, vec2(x, y * 2.0)])
        .map(|[pos, dir]| {
            let body = vec3(
                pos.x * 0.5 + pos.x.signum() * BODY_WIDTH * 0.45,
                0.02,
                pos.y,
            );
            let foot = (dir.normalize_or_zero() * 0.2 + vec2(pos.x, pos.y))
                .extend(0.0)
                .xzy();
            let knee = solve_knee(body, foot, R1, R2);

            CritterLeg {
                local_body: body,
                local_comfy_position: foot,
                comfy_distance: 0.4,
                global_knee: knee,
                global_body: body,
                global_foot: foot,
                global_previous_target: foot,
                global_target: foot,
                t: 0.0,
                animation_speed: 6.,
            }
        })
        .collect_vec()
        .try_into()
        .unwrap();

    commands
        .spawn((
            MaterialMeshBundle {
                mesh: meshes.add(shape::Box::new(0., 0., 0.).into()),
                material,
                transform: Transform::from_translation(vec3(0.0, BODY_CLEAR, 0.0)),
                ..default()
            },
            Critter {
                legs,
                velocity: Vec3::ZERO,
                moving: false,
                set_priorety: false,
                just_moved: false,
            },
        ))
        .id()
}

fn update_critter_mesh(
    mut critters: Query<(&mut Handle<Mesh>, &GlobalTransform, &Critter)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (mesh, transform, cirtter) in critters.iter_mut() {
        let new_mesh = make_hlod_critter_mesh(&cirtter.legs, *transform);
        let mesh_ptr = mesh.into_inner();
        *mesh_ptr = meshes.add(new_mesh);
    }
}
// fn leg_transform_between(a: Vec3, b: Vec3) -> Transform {
//     let mid = (a + b) * 0.5;
//     Transform::from_translation(mid)
//         .looking_at(b, Vec3::Y)
//         .with_scale(vec3(1.0, 1.0, a.distance(b)))
// }

/*
  -- Knee  --
R1| || \\   \
 | ||   \\   \ R2
-- O     \\   \
          \\   \
          Foot --
===========================

     >  Foot >
Old            Target
------- time (t) ----------->
*/
fn coordinate_critter(
    // time: Res<Time>,
    mut critters: Query<(&mut Critter, &GlobalTransform)>,
    // mut transforms: Query<&mut Transform, Without<Critter>>,
) {
    // let delta = time.delta_seconds();
    let mut rng = thread_rng();
    for (mut critter, critter_transform) in critters.iter_mut() {
        let vel = critter.velocity;
        let (set1, set2): (Vec<_>, Vec<_>) = critter
            .legs
            .iter_mut()
            .enumerate()
            .map(|(i, leg)| {
                let global_comfy_postion =
                    critter_transform.transform_point(leg.local_comfy_position);
                let knee_angle = (leg.global_body - leg.global_knee)
                    .normalize_or_zero()
                    .dot((leg.global_foot - leg.global_knee).normalize_or_zero());
                let global_comfy_direction =
                    (global_comfy_postion.xz() - leg.global_body.xz()).normalize_or_zero();
                let leg_angle = global_comfy_direction
                    .dot((leg.global_foot.xz() - leg.global_body.xz()).normalize_or_zero());
                let wants_to_move = knee_angle > 0.85
                    || leg_angle < 0.7
                    || leg.global_body.distance(leg.global_foot) >= R1 + R2;
                // || leg.global_knee.distance(leg.global_body) < 0.01;
                (
                    global_comfy_direction,
                    wants_to_move,
                    i,
                    global_comfy_postion,
                )
            })
            .tuples()
            .unzip();
        let move_leg = |global_comfy_direction,
                        global_comfy_postion,
                        leg: &mut CritterLeg,
                        rng: &mut rand::rngs::ThreadRng| {
            leg.global_previous_target = leg.global_foot;
            leg.global_target = global_comfy_postion
                + vel.clamp_length_max(
                    leg.comfy_distance
                        * (1.0 + 0.5 * vel.normalize().xz().dot(global_comfy_direction)),
                ) * 0.4;
            leg.t = (-rng.gen_range(-0.0..1.0) + vel.length()).min(0.0);
        };

        if critter.moving {
            if set1.iter().all(|(_, _, i, _)| critter.legs[*i].t == 1.0)
                && set2.iter().all(|(_, _, i, _)| critter.legs[*i].t == 1.0)
            {
                critter.moving = false;
                critter.set_priorety = !critter.set_priorety;
                critter.just_moved = true;
            }
        } else if set1.iter().any(|ele| ele.1) && (critter.set_priorety || !critter.just_moved) {
            critter.moving = true;
            for (global_comfy_direction, _wants_to_move, i, global_comfy_postion) in set1 {
                let leg: &mut CritterLeg = &mut critter.legs[i];
                move_leg(global_comfy_direction, global_comfy_postion, leg, &mut rng);
            }
        } else if set2.iter().any(|ele| ele.1) && (!critter.set_priorety || !critter.just_moved) {
            critter.moving = true;
            for (global_comfy_direction, _wants_to_move, i, global_comfy_postion) in set2 {
                let leg: &mut CritterLeg = &mut critter.legs[i];
                move_leg(global_comfy_direction, global_comfy_postion, leg, &mut rng);
            }
        } else {
            critter.just_moved = false;
        }
    }
}

fn update_critter(time: Res<Time>, mut critters: Query<(&mut Critter, &GlobalTransform)>) {
    let delta = time.delta_seconds();
    for (mut critter, critter_transform) in critters.iter_mut() {
        let vel = critter.velocity;
        for mut leg in critter.legs.iter_mut() {
            let body = critter_transform.transform_point(leg.local_body);
            let knee = solve_knee(body, leg.global_foot, R1, R2);

            leg.t = (leg.t + delta * (leg.animation_speed + 1.0 * vel.length())).min(1.0);
            let t = leg.t.max(0.0);
            leg.global_foot = leg
                .global_previous_target
                .lerp(leg.global_target, t * t * (3.0 - 2.0 * t));
            leg.global_foot.y =
                t.mul(t).mul(1.0 - t) * leg.global_previous_target.distance(leg.global_target);
            leg.global_knee = knee;
            leg.global_body = body;
        }
    }
}

fn solve_knee(body: Vec3, foot: Vec3, r1: f32, r2: f32) -> Vec3 {
    let foot = foot - body;
    let foot_xz = foot.xz();
    let foot_xz_len = foot_xz.length();
    let foot_xz_nor = foot_xz / foot_xz_len;
    let Vec2 {
        x: knee_dist,
        y: knee_height,
    } = solve(vec2(foot_xz_len, foot.y), r1, r2);
    vec3(
        foot_xz_nor.x * knee_dist,
        knee_height,
        foot_xz_nor.y * knee_dist,
    ) + body
}

// Adapted from https://iquilezles.org/articles/simpleik/
fn solve(p: Vec2, r1: f32, r2: f32) -> Vec2 {
    let h = p.dot(p);
    let w = h + r1 * r1 - r2 * r2;
    let s = (4.0 * r1 * r1 * h - w * w).max(0.0);
    let result = (w * p + vec2(-p.y, p.x) * s.sqrt()) * 0.5 / h;
    assert_ne!(result.x, f32::NAN);
    assert_ne!(result.x, f32::NAN);
    result
}

fn make_llod_critter_mesh(legs: &[CritterLeg], global_transform: GlobalTransform) -> Mesh {
    let mut pos = Vec::with_capacity(10 * 8);
    let mut nor = Vec::with_capacity(10 * 8);
    let mut indices = Vec::with_capacity(45 * 8);
    let leg_indices = leg_llod_indeices();

    let trans = global_transform.affine().inverse();
    for leg in legs {
        let (more_positions, more_normals) = make_llod_leg_mesh(
            // global_transform.transform_point(leg.global_body),
            // global_transform.transform_point(leg.global_knee),
            // global_transform.transform_point(leg.global_foot),
            trans.transform_point3(leg.global_body),
            trans.transform_point3(leg.global_knee),
            trans.transform_point3(leg.global_foot),
            LEG_WIDTH * 0.5,
        );
        leg_indices
            .iter()
            .map(|i| i + pos.len() as u16)
            .for_each(|i| indices.push(i));
        pos.extend_from_slice(&more_positions);
        nor.extend_from_slice(&more_normals);
    }
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    mesh
}

fn make_hlod_critter_mesh(legs: &[CritterLeg], global_transform: GlobalTransform) -> Mesh {
    let mut pos = Vec::with_capacity(10 * 8);
    let mut nor = Vec::with_capacity(10 * 8);
    let mut indices = Vec::with_capacity(45 * 8);
    let more_indices = leg_llod_indeices();
    let trans = global_transform.affine().inverse();
    for leg in legs {
        let (more_positions, more_normals) = make_llod_leg_mesh(
            // global_transform.transform_point(leg.global_body),
            // global_transform.transform_point(leg.global_knee),
            // global_transform.transform_point(leg.global_foot),
            trans.transform_point3(leg.global_body),
            trans.transform_point3(leg.global_knee),
            trans.transform_point3(leg.global_foot),
            LEG_WIDTH,
            // 3,
            // 100,
            // 100,
        );
        more_indices
            .iter()
            .map(|i| i + pos.len() as u16)
            .for_each(|i| indices.push(i));
        pos.extend_from_slice(&more_positions);
        nor.extend_from_slice(&more_normals);
    }
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U16(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nor);
    mesh
}

/*     5* --- 8*
       | \  k | \
     3|*  *4-6*-7
     |||       \ \\
    2*||        \ \\
    B||          \ \\
   * *            \ \\
  0   1            \ \\
                    \  *11
                     \ |\
                     9*--*10
                       \|/
                        F12
*/
fn leg_llod_indeices() -> Vec<u16> {
    [
        &quad(0, 2, 3, 5)[..],
        &quad(1, 0, 4, 3)[..],
        &quad(2, 1, 5, 4)[..],
        &quad(3, 5, 6, 8)[..],
        &quad(4, 3, 7, 6)[..],
        &quad(5, 4, 8, 7)[..],
        &quad(6, 8, 9, 11)[..],
        &quad(7, 6, 10, 9)[..],
        &quad(8, 7, 11, 10)[..],
        &[11, 9, 12][..],
        &[10, 11, 12][..],
        &[9, 10, 12][..],
    ]
    .concat()
}

fn make_llod_leg_mesh(
    body: Vec3,
    knee: Vec3,
    foot: Vec3,
    thickness: f32,
) -> (Vec<Vec3>, Vec<Vec3>) {
    let leg_dir = (foot - body).xz().extend(0.0).xzy().normalize_or_zero();
    let mut vertices = Vec::with_capacity(10);
    let mut normals = Vec::with_capacity(10);
    for (position, axis) in [
        (body, leg_dir),
        (knee - leg_dir * 0.075, leg_dir),
        (knee - leg_dir * 0.025, (foot - body).normalize_or_zero()),
        (
            foot + vec3(-leg_dir.x, 0.5, -leg_dir.y) * 0.01,
            (foot - knee).normalize_or_zero(),
        ),
    ] {
        for k in 0..3 {
            let theta = k as f32 / 3.0 * TAU;
            let para = axis.cross(Vec3::Y);
            let para = para.cross(axis) * thickness;
            let displace = erot(para, axis, theta);
            normals.push(displace);
            vertices.push(position + displace);
        }
    }
    vertices.push(foot);
    normals.push(foot.normalize());
    (vertices, normals)
}

fn make_hlod_leg_mesh(
    body: Vec3,
    knee: Vec3,
    foot: Vec3,
    thickness: f32,
    segments: usize,
    upper_spacers: usize,
    lower_spacers: usize,
) -> (Vec<Vec3>, Vec<Vec3>, Vec<u16>) {
    let leg_dir = (foot - body).xz().extend(0.0).xzy();
    let num_vetices = segments * (3 + upper_spacers + lower_spacers) + 1;
    let mut positions = Vec::with_capacity(num_vetices);
    let mut normals = Vec::with_capacity(num_vetices);
    let mut indices = Vec::new();

    let mut sector = |position, axis, positions: &mut Vec<Vec3>| {
        for k in 0..segments {
            let theta = k as f32 / 3.0 * TAU;
            let displace = erot(vec3(0.0, thickness, 0.0), axis, theta);
            normals.push(displace);
            positions.push(position + displace);
        }
    };

    let mut sector_indices = |start| {
        // let start = start - 2;
        for edge in (0..=segments).map(|i: usize| -> usize { i + start }) {
            let prev = edge.checked_sub(1).unwrap_or(segments);
            let next = &quad(
                prev as u16,
                edge as u16,
                (prev - segments) as u16,
                (edge - segments) as u16,
            );
            println!("{:?}", next);
            indices.extend_from_slice(next);
        }
    };

    sector(body, leg_dir, &mut positions);
    for d in 1..upper_spacers {
        let t = d as f32 / upper_spacers as f32;
        let h = smoothstep(t) * (knee.y - body.y) + body.y;
        let position = body.xz().lerp(knee.xz(), t).extend(h).xzy();
        // let theta = smoothstep_deriv(t).atan();
        // let axis = erot(leg_dir, perpendicular, theta);
        sector(position, leg_dir, &mut positions);
        sector_indices(positions.len());
    }
    sector(knee, leg_dir, &mut positions);
    sector_indices(positions.len());
    for d in 1..lower_spacers {
        let t = d as f32 / lower_spacers as f32;
        let h = knee.y - 2.0 * (knee.y - foot.y) * smoothstep(t);
        let position = knee.xz().lerp(foot.xz(), t).extend(h).xzy();
        // let theta = smoothstep_deriv(t).atan();
        // let axis = erot(leg_dir, perpendicular, theta);
        sector(position, leg_dir, &mut positions);
        sector_indices(positions.len());
    }
    sector(foot - leg_dir.normalize() * 0.02, leg_dir, &mut positions);
    sector_indices(positions.len());
    positions.push(foot);
    normals.push(leg_dir);

    let last = positions.len();
    for edge in 0..segments {
        let prev = segments.checked_sub(1).unwrap_or(segments);
        indices.extend_from_slice(&[edge as u16, prev as u16, last as u16]);
    }

    (positions, normals, indices)
}

fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn smoothstep_deriv(x: f32) -> f32 {
    6.0 * x * (1.0 - x)
}

fn erot(point: Vec3, axis: Vec3, theta: f32) -> Vec3 {
    let (s, c) = theta.sin_cos();
    (point.dot(axis) * axis).lerp(point, c) + s * axis.cross(point)
}

const fn quad<T: Copy>(a: T, b: T, c: T, d: T) -> [T; 6] {
    // a---c
    // | / |
    // b---d
    [a, c, b, b, c, d]
}
