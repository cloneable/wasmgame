extern crate std;

use std::result::Result;
use std::{panic, unreachable};

pub fn look_at(eye: &Vec3, center: &Vec3, up: &Vec3) -> Mat4 {
    let forward = (center - eye).normalize();
    let side = forward.cross(up).normalize();
    let up = side.cross(&forward);
    Mat4::with_vecs(
        Vec4::new(side.x, up.x, -forward.x, 0.0),
        Vec4::new(side.y, up.y, -forward.y, 0.0),
        Vec4::new(side.z, up.z, -forward.z, 0.0),
        Vec4::new(-side.dot(eye), -up.dot(eye), forward.dot(eye), 1.0),
    )
}

fn radians(degrees: f32) -> f32 {
    degrees * (std::f32::consts::PI / 180.0)
}

pub fn project(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let scale = 1.0 / (radians(fov) / 2.0).tan();
    let d = -1.0 / (far - near);
    Mat4::with_vecs(
        Vec4::new(scale / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, scale, 0.0, 0.0),
        Vec4::new(0.0, 0.0, (far + near) * d, -1.0),
        Vec4::new(0.0, 0.0, 2.0 * far * near * d, 0.0),
    )
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn with(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn normalize(&self) -> Vec3 {
        if self.is_zero() {
            return *self;
        }
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }
}

impl std::ops::Sub for &Vec3 {
    type Output = Vec3;
    fn sub(self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn dot(&self, other: &Vec4) -> f32 {
        self.x * other.x + //br
        self.y * other.y + //br
        self.z * other.z + //br
        self.w * other.w
    }
}

impl std::default::Default for Vec4 {
    fn default() -> Self {
        Vec4::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl std::ops::Index<usize> for Vec4 {
    type Output = f32;
    fn index(&self, i: usize) -> &f32 {
        match i {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => unreachable!(),
        }
    }
}

impl std::ops::IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Debug for Vec4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_tuple("Vec4")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .field(&self.w)
            .finish()
    }
}

#[repr(C)]
pub union Mat4 {
    buf: [f32; 16],
    vecs: [Vec4; 4],
}

impl std::fmt::Debug for Mat4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        unsafe {
            f.debug_tuple("Mat4")
                .field(&self.vecs[0])
                .field(&self.vecs[1])
                .field(&self.vecs[2])
                .field(&self.vecs[3])
                .finish()
        }
    }
}

impl std::cmp::PartialEq for Mat4 {
    fn eq(&self, other: &Mat4) -> bool {
        unsafe { self.buf == other.buf }
    }
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
        buf: [
            1.0, 0.0, 0.0, 0.0, //br
            0.0, 1.0, 0.0, 0.0, //br
            0.0, 0.0, 1.0, 0.0, //br
            0.0, 0.0, 0.0, 1.0, //br
        ],
    };

    pub fn new() -> Self {
        Mat4 { buf: [0.0; 16] }
    }

    pub fn with_vecs(a: Vec4, b: Vec4, c: Vec4, d: Vec4) -> Self {
        let vecs = [a, b, c, d];
        Mat4 { vecs }
    }

    pub fn with_array(buf: [f32; 16]) -> Self {
        Mat4 { buf }
    }

    pub fn column(&self, i: usize) -> Vec4 {
        self[i]
    }

    pub fn row(&self, i: usize) -> Vec4 {
        Vec4::new(self[0][i], self[1][i], self[2][i], self[3][i])
    }
}

impl std::ops::Index<usize> for Mat4 {
    type Output = Vec4;
    fn index(&self, i: usize) -> &Vec4 {
        unsafe { &self.vecs[i] }
    }
}

impl std::ops::IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, i: usize) -> &mut Vec4 {
        unsafe { &mut self.vecs[i] }
    }
}

impl std::ops::Mul<&Mat4> for &Mat4 {
    type Output = Mat4;
    fn mul(self, m: &Mat4) -> Mat4 {
        let mut u = Mat4::new();
        for j in 0..=3 {
            let c = m[j];
            for i in 0..=3 {
                let r = self.row(i);
                u[j][i] = r.dot(&c);
            }
        }
        u
    }
}

#[cfg(test)]
pub mod tests {
    extern crate std;
    extern crate wasm_bindgen_test;
    use std::{assert_eq, panic};
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    #[wasm_bindgen_test]
    fn test_vec4_dot() {
        let v1 = Vec4::new(1.0, 5.0, 9.0, 13.0);
        let v2 = Vec4::new(2.0, 2.0, 2.0, 2.0);
        assert_eq!(v1.dot(&v2), (1 * 2 + 5 * 2 + 9 * 2 + 13 * 2) as f32);
    }

    #[wasm_bindgen_test]
    fn test_mat4_row() {
        let m = Mat4::with_array([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        assert_eq!(m.row(0), Vec4::new(1.0, 5.0, 9.0, 13.0));
        assert_eq!(m.row(1), Vec4::new(2.0, 6.0, 10.0, 14.0));
        assert_eq!(m.row(2), Vec4::new(3.0, 7.0, 11.0, 15.0));
        assert_eq!(m.row(3), Vec4::new(4.0, 8.0, 12.0, 16.0));
    }

    #[wasm_bindgen_test]
    fn test_mat4_union() {
        let m1 = Mat4::with_array([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        let m2 = Mat4::with_vecs(
            Vec4::new(1.0, 2.0, 3.0, 4.0),
            Vec4::new(5.0, 6.0, 7.0, 8.0),
            Vec4::new(9.0, 10.0, 11.0, 12.0),
            Vec4::new(13.0, 14.0, 15.0, 16.0),
        );
        assert_eq!(m1, m2);
    }

    #[wasm_bindgen_test]
    fn test_mat4_mul() {
        let m1 = Mat4::with_array([
            1.0, -2.0, -2.0, -2.0, //br
            2.0, 1.0, -2.0, -2.0, //br
            2.0, 2.0, 1.0, -2.0, //br
            2.0, 2.0, 2.0, 1.0, //br
        ]);
        let m2 = Mat4::with_array([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        let m3 = Mat4::with_array([
            19.0, 14.0, 5.0, -8.0, //br
            47.0, 26.0, 1.0, -28.0, //br
            75.0, 38.0, -3.0, -48.0, //br
            103.0, 50.0, -7.0, -68.0, //br
        ]);
        assert_eq!(&m1 * &m2, m3);
    }
}
