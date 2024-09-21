#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]

use std::{
    array::from_fn,
    thread::sleep,
    time::{Duration, SystemTime},
};

use glam::{vec3, Quat, Vec3};

#[allow(clippy::too_many_lines)]
fn main() {
    const VIEW_SIZE: f32 = 80.;
    const CUBE_SIZE: f32 = 20.;
    const VIEW_DISTANCE: f32 = 70.;
    const CAMERA_POSITION: Vec3 = vec3(0., 40., 0.);
    const SCALE: &str =
        r#"$@B%8&WM#*oahkbdpqwmZO0QLCJUsYXzcvunxrjft/\|()1{}[]?-_+~<>i!lI;:,"^`'. "#;
    let mut axes = [-1., 1.]
        .into_iter()
        .flat_map(|mult| Vec3::AXES.map(|axis| axis * mult));
    let axes: [_; Vec3::AXES.len() * 2] = from_fn(|_| axes.next().unwrap());
    let start = SystemTime::now();
    let mut max = 0.;
    let mut min = 0.;
    loop {
        let relatives = (0..VIEW_SIZE as usize)
            .step_by(2)
            .map(|y| {
                (0..VIEW_SIZE as usize)
                    .map(move |x| {
                        let (x, y) = (x as f32, y as f32);
                        let view_point = vec3(
                            -VIEW_SIZE / 2. + x,
                            CAMERA_POSITION.y - VIEW_DISTANCE,
                            VIEW_SIZE / 2. - y,
                        );
                        let ray = view_point - CAMERA_POSITION;
                        let int = axes
                            .into_iter()
                            .filter_map(|axis| {
                                let rotation =
                                    Quat::from_rotation_z(
                                        SystemTime::now()
                                            .duration_since(start)
                                            .unwrap()
                                            .as_secs_f32()
                                            / 2.,
                                    ) * Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize());
                                let plane_normal = rotation * axis;
                                let plane_point = plane_normal * CUBE_SIZE / 2.;
                                let int =
                                    line_plane_int(CAMERA_POSITION, ray, plane_point, plane_normal)
                                        .collapse();
                                #[allow(clippy::float_cmp)]
                                let mut other_normals =
                                    axes.into_iter().filter(|other| other.dot(axis).abs() != 1.);
                                let other_normals: [_; 4] =
                                    from_fn(|_| other_normals.next().unwrap());
                                let plane_pairs: [[_; 2]; 2] =
                                    [Vec3::eq, Vec3::ne].map(|function| {
                                        let mut pair = other_normals
                                            .into_iter()
                                            .filter(|&other| {
                                                function(&other.abs(), &other_normals[0].abs())
                                            })
                                            .map(|other| {
                                                (
                                                    rotation * other,
                                                    rotation * other * CUBE_SIZE / 2.,
                                                )
                                            });
                                        from_fn(|_| pair.next().unwrap())
                                    });
                                int.and_then(|int| {
                                    plane_pairs
                                        .into_iter()
                                        .all(|pair| {
                                            pair.into_iter().all(|(normal, point)| {
                                                (int - point).dot(normal) < 0.
                                            })
                                        })
                                        .then_some(int)
                                })
                                .map(|int| (int, plane_normal))
                            })
                            .min_by(|(a, _), (b, _)| {
                                f32::partial_cmp(
                                    &a.distance(CAMERA_POSITION),
                                    &b.distance(CAMERA_POSITION),
                                )
                                .unwrap()
                            });
                        int.and_then(|(int, plane_normal)| {
                            ((int - CAMERA_POSITION).normalize().dot(ray) > 0.)
                                .then(|| plane_normal.angle_between(Vec3::ONE))
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        for (function, default, extreme, compare) in [
            (
                Iterator::max_by as fn(_, _) -> Option<_>,
                1.,
                &mut max,
                f32::max as fn(f32, f32) -> f32,
            ),
            (
                Iterator::min_by as fn(_, _) -> Option<_>,
                0.,
                &mut min,
                f32::min as fn(f32, f32) -> f32,
            ),
        ] {
            *extreme = compare(
                *extreme,
                function(
                    relatives.iter().flatten().filter_map(|relative| *relative),
                    |a: &f32, b: &f32| a.partial_cmp(b).unwrap(),
                )
                .unwrap_or(default),
            );
        }
        println!(
            "{}",
            relatives
                .into_iter()
                .map(|row| row
                    .into_iter()
                    .map(|relative| SCALE
                        .chars()
                        .nth(relative.map_or_else(
                            || SCALE.len() as f32 - 1.,
                            |point| (point - min) / (max - min) * (SCALE.len() as f32 - 1.)
                        ) as usize)
                        .unwrap())
                    .collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
        );
        sleep(Duration::from_secs_f32(1. / 60.));
    }
}

enum LinePlaneIntResult {
    None,
    Point(Vec3),
    Line { line_point: Vec3 },
}

impl LinePlaneIntResult {
    const fn collapse(self) -> Option<Vec3> {
        match self {
            Self::None => None,
            Self::Point(point) => Some(point),
            Self::Line { line_point, .. } => Some(line_point),
        }
    }
}

fn line_plane_int(
    line_point: Vec3,
    line_direction: Vec3,
    plane_point: Vec3,
    plane_normal: Vec3,
) -> LinePlaneIntResult {
    let line_direction = line_direction.normalize();
    let parallel = line_direction.dot(plane_normal) == 0.;
    if parallel {
        if (plane_point - line_point).dot(plane_normal) == 0. {
            return LinePlaneIntResult::Line { line_point };
        }
        return LinePlaneIntResult::None;
    }
    let parameter = (plane_point - line_point).dot(plane_normal) / line_direction.dot(plane_normal);
    let int = line_point + line_direction * parameter;
    LinePlaneIntResult::Point(int)
}
