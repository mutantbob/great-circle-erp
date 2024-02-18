use cgmath::{InnerSpace, Matrix3, SquareMatrix, Vector3};
use egui::Vec2;
use std::f32::consts::{PI, TAU};

pub struct GreatCircleRemapper {
    pub matrix: Matrix3<f32>,
    pub inverse: Matrix3<f32>,
}

impl GreatCircleRemapper {
    pub(crate) fn new(anchors: &[Vec2]) -> Self {
        let matrix = Self::matrix_from_anchors(anchors);

        let rval = Self {
            matrix,
            inverse: matrix.invert().unwrap(),
        };

        if true {
            for anchor in anchors {
                let twisted = rval.twist(anchor.x, anchor.y);
                println!("{:?} -> {:?}", anchor, twisted);
                println!("{:?} <- {:?}", rval.untwist(twisted.x, twisted.y), twisted);
            }
        }

        rval
    }

    pub fn matrix_from_anchors(anchors: &[Vec2]) -> Matrix3<f32> {
        match anchors.len() {
            0 => Matrix3::identity(),
            1 => {
                let axis_x = spherical_to_cartesian(fracv_to_radians(anchors[0]));
                let axis_y = Vector3::cross(Vector3::new(0.0, 0.0, 1.0), axis_x).normalize();
                let axis_z = axis_x.cross(axis_y).normalize();
                println!("new axes {:.3?}, {:.3?}, {:.3?}", axis_x, axis_y, axis_z);
                Matrix3::from_cols(axis_x, axis_y, axis_z)
            }
            _ => {
                let anchor1 = anchors[0];
                let anchor2 = anchors[1];
                let anchor1 = spherical_to_cartesian(fracv_to_radians(anchor1));
                let anchor2 = spherical_to_cartesian(fracv_to_radians(anchor2));
                let axis_x = ((anchor1 + anchor2) * 0.5).normalize();
                let axis_z = anchor1.cross(anchor2).normalize();
                let axis_y = axis_z.cross(axis_x).normalize();
                Matrix3::from_cols(axis_x, axis_y, axis_z)
            }
        }
    }

    pub(crate) fn twist(&self, longitude_frac: f32, latitude_frac: f32) -> Vec2 {
        let theta_phi = frac_to_radians(longitude_frac, latitude_frac);

        let xyz = spherical_to_cartesian(theta_phi);

        cartesian_to_lat_long(self.inverse * xyz)
    }

    pub(crate) fn untwist(&self, longitude_frac: f32, latitude_frac: f32) -> Vec2 {
        let theta_phi = frac_to_radians(longitude_frac, latitude_frac);

        let xyz = spherical_to_cartesian(theta_phi);

        cartesian_to_lat_long(self.matrix * xyz)
    }
}

pub fn transform_ll_to_ll(longitude_frac: f32, latitude_frac: f32, matrix: &Matrix3<f32>) -> Vec2 {
    let theta_phi = frac_to_radians(longitude_frac, latitude_frac);

    let xyz = spherical_to_cartesian(theta_phi);

    cartesian_to_lat_long(matrix * xyz)
}

fn cartesian_to_lat_long(xyz: Vector3<f32>) -> Vec2 {
    let r = Vec2::new(xyz.x, xyz.y).length();
    let phi = f32::atan2(xyz.z, r);
    let theta = f32::atan2(-xyz.y, -xyz.x);
    let v = phi / PI + 0.5;
    let u = theta / TAU;

    Vec2::new(my_fmod(u, 1.0), my_fmod(v, 1.0))
}

fn my_fmod(a: f32, b: f32) -> f32 {
    let rval = a % b;
    if a < 0.0 {
        rval + b
    } else {
        rval
    }
}

fn frac_to_radians(longitude_frac: f32, latitude_frac: f32) -> Vec2 {
    let theta = longitude_frac * TAU;
    let phi = (latitude_frac - 0.5) * PI;
    Vec2::new(theta, phi)
}

fn fracv_to_radians(uv: Vec2) -> Vec2 {
    frac_to_radians(uv.x, uv.y)
}

fn spherical_to_cartesian(Vec2 { x: theta, y: phi }: Vec2) -> Vector3<f32> {
    let z = phi.sin();
    let r = phi.cos();
    let x = -theta.cos() * r;
    let y = -theta.sin() * r;

    let xyz: Vector3<f32> = Vector3::new(x, y, z);
    xyz
}
