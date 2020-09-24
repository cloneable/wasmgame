extern crate std;

use std::convert::From;
use std::option::{Option, Option::None, Option::Some};
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

impl std::convert::From<[f32; 3]> for Vec3 {
    fn from(buf: [f32; 3]) -> Vec3 {
        Vec3 {
            x: buf[0],
            y: buf[1],
            z: buf[2],
        }
    }
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

    pub fn apply(&self, func: fn(f32) -> f32) -> Vec3 {
        Vec3 {
            x: func(self.x),
            y: func(self.y),
            z: func(self.z),
        }
    }
}

impl std::ops::AddAssign<Vec3> for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::AddAssign<&Vec3> for Vec3 {
    fn add_assign(&mut self, other: &Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl std::ops::SubAssign<Vec3> for Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl std::ops::SubAssign<&Vec3> for Vec3 {
    fn sub_assign(&mut self, other: &Vec3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
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

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
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

    pub fn rgb(x: u8, y: u8, z: u8) -> Self {
        Vec4 {
            x: x as f32 / 255.0,
            y: y as f32 / 255.0,
            z: z as f32 / 255.0,
            w: 1.0,
        }
    }

    pub fn from_vec3(v: Vec3, w: f32) -> Self {
        Vec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
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
        f.debug_list()
            .entry(&self.x)
            .entry(&self.y)
            .entry(&self.z)
            .entry(&self.w)
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
        unsafe { f.debug_list().entries(&self.vecs).finish() }
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

    pub fn slice(&self) -> &[f32] {
        unsafe { &self.buf }
    }

    pub fn column(&self, i: usize) -> Vec4 {
        self[i]
    }

    pub fn row(&self, i: usize) -> Vec4 {
        Vec4::new(self[0][i], self[1][i], self[2][i], self[3][i])
    }

    pub fn transpose(&self) -> Mat4 {
        Mat4::with_vecs(self.row(0), self.row(1), self.row(2), self.row(3))
    }

    pub fn to_3x3(&self) -> Mat4 {
        let mut buf: [f32; 16] = [0.0; 16];
        unsafe {
            for c in 0..=2 {
                for r in 0..=2 {
                    buf[c * 4 + r] = self.buf[c * 4 + r];
                }
            }
        }
        Mat4::with_array(buf)
    }

    pub fn invert(&self) -> Option<Mat4> {
        let mut i: [f32; 16] = [0.0; 16];
        let m = unsafe { &self.buf };

        // Mesa's gluInvertMatrix using the adjugate matrix.
        // TODO: Rebuild this macros to make it more readable.
        i[0] = m[5]  * m[10] * m[15] - //br
             m[5]  * m[11] * m[14] - //br
             m[9]  * m[6]  * m[15] + //br
             m[9]  * m[7]  * m[14] +//br
             m[13] * m[6]  * m[11] - //br
             m[13] * m[7]  * m[10];

        i[4] = -m[4]  * m[10] * m[15] + //br
              m[4]  * m[11] * m[14] + //br
              m[8]  * m[6]  * m[15] - //br
              m[8]  * m[7]  * m[14] - //br
              m[12] * m[6]  * m[11] + //br
              m[12] * m[7]  * m[10];

        i[8] = m[4]  * m[9] * m[15] - //br
             m[4]  * m[11] * m[13] - //br
             m[8]  * m[5] * m[15] + //br
             m[8]  * m[7] * m[13] + //br
             m[12] * m[5] * m[11] - //br
             m[12] * m[7] * m[9];

        i[12] = -m[4]  * m[9] * m[14] + //br
               m[4]  * m[10] * m[13] +//br
               m[8]  * m[5] * m[14] - //br
               m[8]  * m[6] * m[13] - //br
               m[12] * m[5] * m[10] + //br
               m[12] * m[6] * m[9];

        i[1] = -m[1]  * m[10] * m[15] + //br
              m[1]  * m[11] * m[14] + //br
              m[9]  * m[2] * m[15] - //br
              m[9]  * m[3] * m[14] - //br
              m[13] * m[2] * m[11] + //br
              m[13] * m[3] * m[10];

        i[5] = m[0]  * m[10] * m[15] - //br
             m[0]  * m[11] * m[14] - //br
             m[8]  * m[2] * m[15] + //br
             m[8]  * m[3] * m[14] + //br
             m[12] * m[2] * m[11] - //br
             m[12] * m[3] * m[10];

        i[9] = -m[0]  * m[9] * m[15] + //br
              m[0]  * m[11] * m[13] + //br
              m[8]  * m[1] * m[15] - //br
              m[8]  * m[3] * m[13] - //br
              m[12] * m[1] * m[11] + //br
              m[12] * m[3] * m[9];

        i[13] = m[0]  * m[9] * m[14] - //br
              m[0]  * m[10] * m[13] - //br
              m[8]  * m[1] * m[14] + //br
              m[8]  * m[2] * m[13] + //br
              m[12] * m[1] * m[10] - //br
              m[12] * m[2] * m[9];

        i[2] = m[1]  * m[6] * m[15] - //br
             m[1]  * m[7] * m[14] - //br
             m[5]  * m[2] * m[15] + //br
             m[5]  * m[3] * m[14] + //br
             m[13] * m[2] * m[7] - //br
             m[13] * m[3] * m[6];

        i[6] = -m[0]  * m[6] * m[15] + //br
              m[0]  * m[7] * m[14] + //br
              m[4]  * m[2] * m[15] - //br
              m[4]  * m[3] * m[14] - //br
              m[12] * m[2] * m[7] + //br
              m[12] * m[3] * m[6];

        i[10] = m[0]  * m[5] * m[15] - //br
              m[0]  * m[7] * m[13] - //br
              m[4]  * m[1] * m[15] + //br
              m[4]  * m[3] * m[13] + //br
              m[12] * m[1] * m[7] - //br
              m[12] * m[3] * m[5];

        i[14] = -m[0]  * m[5] * m[14] + //br
               m[0]  * m[6] * m[13] + //br
               m[4]  * m[1] * m[14] - //br
               m[4]  * m[2] * m[13] - //br
               m[12] * m[1] * m[6] + //br
               m[12] * m[2] * m[5];

        i[3] = -m[1] * m[6] * m[11] + //br
              m[1] * m[7] * m[10] + //br
              m[5] * m[2] * m[11] - //br
              m[5] * m[3] * m[10] - //br
              m[9] * m[2] * m[7] + //br
              m[9] * m[3] * m[6];

        i[7] = m[0] * m[6] * m[11] - //br
             m[0] * m[7] * m[10] - //br
             m[4] * m[2] * m[11] + //br
             m[4] * m[3] * m[10] + //br
             m[8] * m[2] * m[7] - //br
             m[8] * m[3] * m[6];

        i[11] = -m[0] * m[5] * m[11] + //br
               m[0] * m[7] * m[9] + //br
               m[4] * m[1] * m[11] - //br
               m[4] * m[3] * m[9] - //br
               m[8] * m[1] * m[7] + //br
               m[8] * m[3] * m[5];

        i[15] = m[0] * m[5] * m[10] - //br
              m[0] * m[6] * m[9] - //br
              m[4] * m[1] * m[10] + //br
              m[4] * m[2] * m[9] + //br
              m[8] * m[1] * m[6] - //br
              m[8] * m[2] * m[5];

        let det = m[0] * i[0] + m[1] * i[4] + m[2] * i[8] + m[3] * i[12];
        if det == 0.0 {
            return None;
        }
        let det = 1.0 / det;
        for t in 0..=15 {
            i[t] *= det;
        }
        Some(Mat4 { buf: i })
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

impl std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;
    fn mul(self, v: Vec4) -> Vec4 {
        let m = &self;
        Vec4 {
            x: m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0] * v[3],
            y: m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1] * v[3],
            z: m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2] * v[3],
            w: m[0][3] * v[0] + m[1][3] * v[1] + m[2][3] * v[2] + m[3][3] * v[3],
        }
    }
}

impl std::ops::Mul<&Vec4> for &Mat4 {
    type Output = Vec4;
    fn mul(self, v: &Vec4) -> Vec4 {
        let m = &self;
        Vec4 {
            x: m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0] * v[3],
            y: m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1] * v[3],
            z: m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2] * v[3],
            w: m[0][3] * v[0] + m[1][3] * v[1] + m[2][3] * v[2] + m[3][3] * v[3],
        }
    }
}

impl std::convert::From<[f32; 16]> for Mat4 {
    fn from(buf: [f32; 16]) -> Self {
        Mat4 { buf }
    }
}

pub struct Quaternion {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

impl Quaternion {
    pub fn new(angles: Vec3) -> Self {
        let rad = angles * (std::f32::consts::PI / 180.0 * 0.5);
        let c = rad.apply(f32::cos);
        let s = rad.apply(f32::sin);
        Quaternion {
            w: c.x * c.y * c.z + s.x * s.y * s.z,
            x: s.x * c.y * c.z - c.x * s.y * s.z,
            y: c.x * s.y * c.z + s.x * c.y * s.z,
            z: c.x * c.y * s.z - s.x * s.y * c.z,
        }
    }
}

impl From<Quaternion> for Mat4 {
    fn from(q: Quaternion) -> Self {
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xz = q.x * q.z;
        let xy = q.x * q.y;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        let a = Vec4::new(
            1.0 - 2.0 * (yy + zz), //br
            2.0 * (xy + wz),       //br
            2.0 * (xz - wy),       //br
            0.0,
        );
        let b = Vec4::new(
            2.0 * (xy - wz),       //br
            1.0 - 2.0 * (xx + zz), //br
            2.0 * (yz + wx),       //br
            0.0,
        );
        let c = Vec4::new(
            2.0 * (xz + wy),       //br
            2.0 * (yz - wx),       //br
            1.0 - 2.0 * (xx + yy), //br
            0.0,
        );
        let d = Vec4::new(0.0, 0.0, 0.0, 1.0);

        Mat4::with_vecs(a, b, c, d)
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
