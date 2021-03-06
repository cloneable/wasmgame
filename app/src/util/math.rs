use ::std::{
    convert::{From, Into},
    option::{
        Option,
        Option::{None, Some},
    },
    result::Result,
};

#[inline(never)]
pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    let forward = (center - eye).normalize();
    let side = forward.cross(up).normalize();
    let up = side.cross(forward);
    Mat4::from([
        [side.x, up.x, -forward.x, 0.0],
        [side.y, up.y, -forward.y, 0.0],
        [side.z, up.z, -forward.z, 0.0],
        [-side.dot(eye), -up.dot(eye), forward.dot(eye), 1.0],
    ])
}

#[inline(never)]
pub fn project(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let scale = 1.0 / (fov.to_radians() / 2.0).tan();
    let d = -1.0 / (far - near);
    Mat4::from([
        [scale / aspect, 0.0, 0.0, 0.0],
        [0.0, scale, 0.0, 0.0],
        [0.0, 0.0, (far + near) * d, -1.0],
        [0.0, 0.0, 2.0 * far * near * d, 0.0],
    ])
}

#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline(always)]
    pub fn new() -> Self {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline(always)]
    pub fn with(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    #[inline(always)]
    pub fn with_rgb(x: u8, y: u8, z: u8) -> Self {
        Vec3 {
            x: x as f32 / 255.0,
            y: y as f32 / 255.0,
            z: z as f32 / 255.0,
        }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.dot(*self).sqrt()
    }

    #[inline(always)]
    pub fn cross(&self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline(always)]
    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Hadamard product
    #[inline(always)]
    pub fn componentwise(&mut self, v: Vec3) -> Vec3 {
        self.combine(v, |a, b| a * b)
    }

    #[inline(always)]
    pub fn normalize(&self) -> Vec3 {
        if self.is_zero() {
            return *self;
        }
        let length = self.length();
        Vec3 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        }
    }

    #[inline(always)]
    pub fn apply(&self, func: fn(f32) -> f32) -> Vec3 {
        Vec3 {
            x: func(self.x),
            y: func(self.y),
            z: func(self.z),
        }
    }

    #[inline(always)]
    pub fn combine(&self, v: Vec3, func: fn(f32, f32) -> f32) -> Vec3 {
        Vec3 {
            x: func(self.x, v.x),
            y: func(self.y, v.y),
            z: func(self.z, v.z),
        }
    }

    pub fn to_polar(&self) -> Vec3 {
        let r = self.length();
        if r == 0.0 {
            return Vec3::new();
        }
        let t = self.x.atan2(self.z).to_degrees();
        let p = (self.y / r).asin().to_degrees();
        Vec3::with(r, t, p)
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let r = self.x;
        let (t_sin, t_cos) = self.y.to_radians().sin_cos();
        let (p_sin, p_cos) = self.z.to_radians().sin_cos();
        Vec3::with(
            r * t_sin * p_cos, //br
            r * p_sin,         //br
            r * t_cos * p_cos,
        )
    }
}

impl ::std::default::Default for Vec3 {
    #[inline(always)]
    fn default() -> Self {
        Vec3::new()
    }
}

impl ::std::fmt::Debug for Vec3 {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        f.debug_list()
            .entry(&self.x)
            .entry(&self.y)
            .entry(&self.z)
            .finish()
    }
}

impl ::std::convert::From<[f32; 3]> for Vec3 {
    #[inline(always)]
    fn from(buf: [f32; 3]) -> Vec3 {
        Vec3 {
            x: buf[0],
            y: buf[1],
            z: buf[2],
        }
    }
}

impl ::std::ops::AddAssign<Vec3> for Vec3 {
    #[inline(always)]
    fn add_assign(&mut self, other: Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl ::std::ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl ::std::ops::SubAssign<Vec3> for Vec3 {
    #[inline(always)]
    fn sub_assign(&mut self, other: Vec3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl ::std::ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl ::std::ops::MulAssign<Vec3> for Vec3 {
    #[inline(always)]
    fn mul_assign(&mut self, v: Vec3) {
        self.x *= v.x;
        self.y *= v.y;
        self.z *= v.z;
    }
}

impl ::std::ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * v.x,
            y: self.y * v.y,
            z: self.z * v.z,
        }
    }
}

impl ::std::ops::MulAssign<f32> for Vec3 {
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl ::std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl ::std::ops::Mul<Vec3> for f32 {
    type Output = Vec3;
    #[inline(always)]
    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self * v.x,
            y: self * v.y,
            z: self * v.z,
        }
    }
}

impl ::std::ops::DivAssign<Vec3> for Vec3 {
    #[inline(always)]
    fn div_assign(&mut self, v: Vec3) {
        self.x /= v.x;
        self.y /= v.y;
        self.z /= v.z;
    }
}

impl ::std::ops::Div<Vec3> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn div(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self.x / v.x,
            y: self.y / v.y,
            z: self.z / v.z,
        }
    }
}

impl ::std::ops::DivAssign<f32> for Vec3 {
    #[inline(always)]
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
    }
}

impl ::std::ops::Div<f32> for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn div(self, scalar: f32) -> Vec3 {
        Vec3 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl ::std::ops::Div<Vec3> for f32 {
    type Output = Vec3;
    #[inline(always)]
    fn div(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self / v.x,
            y: self / v.y,
            z: self / v.z,
        }
    }
}

impl ::std::ops::Neg for Vec3 {
    type Output = Vec3;
    #[inline(always)]
    fn neg(self) -> Vec3 {
        Vec3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C, align(16))]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[inline(always)]
    pub fn new() -> Self {
        Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }

    #[inline(always)]
    pub fn with(x: f32, y: f32, z: f32, w: f32) -> Self {
        Vec4 { x, y, z, w }
    }

    #[inline(always)]
    pub fn with_xyz(x: f32, y: f32, z: f32) -> Self {
        Vec4::with(x, y, z, 1.0)
    }

    #[inline(always)]
    pub fn with_vec3(v: Vec3, w: f32) -> Self {
        Vec4::with(v.x, v.y, v.z, w)
    }

    #[inline(always)]
    pub fn with_rgb(x: u8, y: u8, z: u8) -> Self {
        Vec4 {
            x: x as f32 / 255.0,
            y: y as f32 / 255.0,
            z: z as f32 / 255.0,
            w: 1.0,
        }
    }

    #[inline(always)]
    pub fn xyz(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    #[inline(always)]
    pub fn from_vec3(v: Vec3, w: f32) -> Self {
        Vec4 {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }

    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0 && self.w == 0.0
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        self.dot(*self).sqrt()
    }

    #[inline(always)]
    pub fn dot(&self, other: Vec4) -> f32 {
        self.x * other.x + //br
        self.y * other.y + //br
        self.z * other.z + //br
        self.w * other.w
    }

    #[inline(always)]
    pub fn normalize(&self) -> Vec4 {
        if self.is_zero() {
            return *self;
        }
        let length = self.length();
        Vec4 {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
            w: self.w / length,
        }
    }

    #[inline(always)]
    pub fn apply(&self, func: fn(f32) -> f32) -> Vec4 {
        Vec4 {
            x: func(self.x),
            y: func(self.y),
            z: func(self.z),
            w: func(self.w),
        }
    }
}

impl ::std::default::Default for Vec4 {
    #[inline(always)]
    fn default() -> Self {
        Vec4::new()
    }
}

impl ::std::convert::From<(Vec3, f32)> for Vec4 {
    #[inline(always)]
    fn from(v: (Vec3, f32)) -> Vec4 {
        Vec4 {
            x: v.0.x,
            y: v.0.y,
            z: v.0.z,
            w: v.1,
        }
    }
}

impl ::std::fmt::Debug for Vec4 {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        f.debug_list()
            .entry(&self.x)
            .entry(&self.y)
            .entry(&self.z)
            .entry(&self.w)
            .finish()
    }
}

impl ::std::ops::AddAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn add_assign(&mut self, other: Vec4) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
        self.w += other.w;
    }
}

impl ::std::ops::Add<Vec4> for Vec4 {
    type Output = Vec4;
    #[inline(always)]
    fn add(self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl ::std::ops::SubAssign<Vec4> for Vec4 {
    #[inline(always)]
    fn sub_assign(&mut self, other: Vec4) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
        self.w -= other.w;
    }
}

impl ::std::ops::Sub<Vec4> for Vec4 {
    type Output = Vec4;
    #[inline(always)]
    fn sub(self, other: Vec4) -> Vec4 {
        Vec4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl ::std::ops::MulAssign<f32> for Vec4 {
    #[inline(always)]
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
        self.w *= scalar;
    }
}

impl ::std::ops::Mul<f32> for Vec4 {
    type Output = Vec4;
    #[inline(always)]
    fn mul(self, scalar: f32) -> Vec4 {
        Vec4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl ::std::ops::DivAssign<f32> for Vec4 {
    #[inline(always)]
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
        self.w /= scalar;
    }
}

impl ::std::ops::Div<f32> for Vec4 {
    type Output = Vec4;
    #[inline(always)]
    fn div(self, scalar: f32) -> Vec4 {
        Vec4 {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
            w: self.w / scalar,
        }
    }
}

impl ::std::ops::Neg for Vec4 {
    type Output = Vec4;
    #[inline(always)]
    fn neg(self) -> Vec4 {
        Vec4 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

#[repr(C, align(16))]
pub union Mat4 {
    buf: [f32; 16],
    columns: [[f32; 4]; 4],
}

impl Mat4 {
    #[inline(always)]
    pub fn identity() -> Self {
        Mat4 {
            buf: [
                1.0, 0.0, 0.0, 0.0, //br
                0.0, 1.0, 0.0, 0.0, //br
                0.0, 0.0, 1.0, 0.0, //br
                0.0, 0.0, 0.0, 1.0, //br
            ],
        }
    }

    #[inline(always)]
    pub fn scaling(v: Vec3) -> Self {
        Mat4 {
            buf: [
                v.x, 0.0, 0.0, 0.0, //br
                0.0, v.y, 0.0, 0.0, //br
                0.0, 0.0, v.z, 0.0, //br
                0.0, 0.0, 0.0, 1.0, //br
            ],
        }
    }

    #[inline(always)]
    pub fn translation(v: Vec3) -> Self {
        Mat4 {
            buf: [
                1.0, 0.0, 0.0, 0.0, //br
                0.0, 1.0, 0.0, 0.0, //br
                0.0, 0.0, 1.0, 0.0, //br
                v.x, v.y, v.z, 1.0, //br
            ],
        }
    }

    #[inline(always)]
    pub fn rotation(v: Vec3) -> Self {
        Quat::rotation(v).into()
    }

    #[inline(always)]
    pub fn slice(&self) -> &[f32] {
        #[allow(unsafe_code)]
        unsafe {
            &self.buf
        }
    }

    #[inline(always)]
    fn buf_mut(&mut self) -> &mut [f32; 16] {
        #[allow(unsafe_code)]
        unsafe {
            &mut self.buf
        }
    }

    #[inline(always)]
    fn row(&self, i: usize) -> [f32; 4] {
        [self[(0, i)], self[(1, i)], self[(2, i)], self[(3, i)]]
    }

    #[inline(always)]
    fn swap(buf: &mut [f32; 16], c: usize, r: usize) {
        let i = c * 4 + r;
        let j = r * 4 + c;
        let tmp = buf[i];
        buf[i] = buf[j];
        buf[j] = tmp;
    }

    #[inline(never)]
    pub fn transpose(&mut self) {
        let buf = self.buf_mut();
        for c in 0..4 {
            for r in 0..c {
                Mat4::swap(buf, c, r);
            }
        }
    }

    #[inline(always)]
    pub fn scale(&mut self, v: Vec3) {
        self[(0, 0)] *= v.x;
        self[(1, 1)] *= v.y;
        self[(2, 2)] *= v.z;
    }

    #[inline(always)]
    pub fn translate(&mut self, v: Vec3) {
        self[(3, 0)] += v.x;
        self[(3, 1)] += v.y;
        self[(3, 2)] += v.z;
    }

    #[inline(never)]
    pub fn to_3x3(&self) -> Mat4 {
        let mut buf: [f32; 16] = [0.0; 16];
        // TODO: Unroll loops.
        for c in 0..=2 {
            for r in 0..=2 {
                #[allow(unsafe_code)]
                {
                    buf[c * 4 + r] = unsafe { self.buf[c * 4 + r] };
                }
            }
        }
        Mat4::from(buf)
    }

    #[inline(never)]
    pub fn invert(&self) -> Option<Mat4> {
        let mut i: [f32; 16] = [0.0; 16];
        #[allow(unsafe_code)]
        let m = unsafe { &self.buf };

        // Mesa's gluInvertMatrix using the adjugate matrix.
        // TODO: Rebuild this with macros to make it more readable.
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
        for v in &mut i {
            *v *= det;
        }
        Some(Mat4 { buf: i })
    }
}

impl ::std::default::Default for Mat4 {
    #[inline(always)]
    fn default() -> Self {
        Mat4::identity()
    }
}

impl ::std::clone::Clone for Mat4 {
    #[inline(always)]
    fn clone(&self) -> Self {
        Mat4 {
            buf: unsafe { self.buf.clone() },
        }
    }
}

impl ::std::fmt::Debug for Mat4 {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        #[allow(unsafe_code)]
        unsafe {
            f.debug_list().entries(&self.columns).finish()
        }
    }
}

impl ::std::ops::Index<(usize, usize)> for Mat4 {
    type Output = f32;
    #[inline(always)]
    fn index(&self, cr: (usize, usize)) -> &f32 {
        #[allow(unsafe_code)]
        unsafe {
            &self.buf[cr.0 * 4 + cr.1]
        }
    }
}

impl ::std::ops::IndexMut<(usize, usize)> for Mat4 {
    #[inline(always)]
    fn index_mut(&mut self, cr: (usize, usize)) -> &mut f32 {
        #[allow(unsafe_code)]
        unsafe {
            &mut self.buf[cr.0 * 4 + cr.1]
        }
    }
}

#[inline(always)]
fn colrowdot(m1: &[f32; 16], m2: &[f32; 16], c: usize, r: usize) -> f32 {
    m1[c * 4].mul_add(
        m2[r],
        m1[c * 4 + 1].mul_add(
            m2[r + 4],
            m1[c * 4 + 2].mul_add(m2[r + 8], m1[c * 4 + 3] * m2[r + 12]),
        ),
    )
    // m1[c * 4] * m2[r]
    //     + m1[c * 4 + 1] * m2[r + 4]
    //     + m1[c * 4 + 2] * m2[r + 8]
    //     + m1[c * 4 + 3] * m2[r + 12]
}

impl ::std::ops::Mul<&Mat4> for &Mat4 {
    type Output = Mat4;
    #[inline(never)]
    fn mul(self, m: &Mat4) -> Mat4 {
        let u = unsafe { &self.buf };
        let v = unsafe { &m.buf };
        Mat4 {
            buf: [
                colrowdot(v, u, 0, 0),
                colrowdot(v, u, 0, 1),
                colrowdot(v, u, 0, 2),
                colrowdot(v, u, 0, 3),
                colrowdot(v, u, 1, 0),
                colrowdot(v, u, 1, 1),
                colrowdot(v, u, 1, 2),
                colrowdot(v, u, 1, 3),
                colrowdot(v, u, 2, 0),
                colrowdot(v, u, 2, 1),
                colrowdot(v, u, 2, 2),
                colrowdot(v, u, 2, 3),
                colrowdot(v, u, 3, 0),
                colrowdot(v, u, 3, 1),
                colrowdot(v, u, 3, 2),
                colrowdot(v, u, 3, 3),
            ],
        }
    }
}

impl ::std::ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;
    #[inline(always)]
    fn mul(self, m: Mat4) -> Mat4 {
        &self * &m
    }
}

impl ::std::ops::Mul<Vec4> for Mat4 {
    type Output = Vec4;
    #[inline(never)]
    fn mul(self, v: Vec4) -> Vec4 {
        let m = &self;
        Vec4 {
            x: m[(0, 0)] * v.x
                + m[(1, 0)] * v.y
                + m[(2, 0)] * v.z
                + m[(3, 0)] * v.w,
            y: m[(0, 1)] * v.x
                + m[(1, 1)] * v.y
                + m[(2, 1)] * v.z
                + m[(3, 1)] * v.w,
            z: m[(0, 2)] * v.x
                + m[(1, 2)] * v.y
                + m[(2, 2)] * v.z
                + m[(3, 2)] * v.w,
            w: m[(0, 3)] * v.x
                + m[(1, 3)] * v.y
                + m[(2, 3)] * v.z
                + m[(3, 3)] * v.w,
        }
    }
}

impl ::std::convert::From<[f32; 16]> for Mat4 {
    #[inline(always)]
    fn from(buf: [f32; 16]) -> Self {
        Mat4 { buf }
    }
}

impl ::std::convert::From<[[f32; 4]; 4]> for Mat4 {
    #[inline(always)]
    fn from(columns: [[f32; 4]; 4]) -> Self {
        Mat4 { columns }
    }
}

#[derive(Copy, Clone)]
struct Quat {
    w: f32,
    x: f32,
    y: f32,
    z: f32,
}

const HALF_PI_RAD: f32 = (::std::f64::consts::PI / 180.0 * 0.5) as f32;

impl Quat {
    #[inline(always)]
    fn identity() -> Self {
        Quat {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    #[inline(never)]
    fn rotation(angles: Vec3) -> Self {
        let rad = angles * HALF_PI_RAD;
        let c = rad.apply(f32::cos);
        let s = rad.apply(f32::sin);
        Quat {
            w: c.x * c.y * c.z + s.x * s.y * s.z,
            x: s.x * c.y * c.z - c.x * s.y * s.z,
            y: c.x * s.y * c.z + s.x * c.y * s.z,
            z: c.x * c.y * s.z - s.x * s.y * c.z,
        }
    }

    #[inline(always)]
    fn from_to(from: Vec3, to: Vec3) -> Self {
        let v = from.cross(to);
        Quat {
            w: 1.0 + from.dot(to),
            x: v.x,
            y: v.y,
            z: v.z,
        }
        .normalize()
    }

    #[inline(always)]
    fn length(&self) -> f32 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z)
            .sqrt()
    }

    #[inline(always)]
    fn normalize(&self) -> Quat {
        let len = self.length();
        Quat {
            w: self.w / len,
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    #[inline(always)]
    fn invert(&self) -> Quat {
        Quat {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ::std::fmt::Display for Quat {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        f.debug_list()
            .entry(&self.w)
            .entry(&self.x)
            .entry(&self.y)
            .entry(&self.z)
            .finish()
    }
}

impl ::std::fmt::Debug for Quat {
    fn fmt(
        &self, f: &mut ::std::fmt::Formatter<'_>,
    ) -> Result<(), ::std::fmt::Error> {
        f.debug_tuple("Quat")
            .field(&self.w)
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}

impl From<Vec3> for Quat {
    #[inline(always)]
    fn from(v: Vec3) -> Self {
        Quat {
            w: 0.0,
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl ::std::ops::Mul<Quat> for Quat {
    type Output = Quat;
    #[inline(never)]
    fn mul(self, q: Quat) -> Quat {
        Quat {
            w: self.w * q.w - self.x * q.w - self.y * q.w - self.z * q.w,
            x: self.w * q.x + self.x * q.x - self.y * q.x + self.z * q.x,
            y: self.w * q.y + self.x * q.y + self.y * q.y - self.z * q.y,
            z: self.w * q.z - self.x * q.z + self.y * q.z + self.z * q.z,
        }
    }
}

impl From<Quat> for Vec3 {
    #[inline(always)]
    fn from(q: Quat) -> Vec3 {
        Vec3 {
            x: q.x,
            y: q.y,
            z: q.z,
        }
    }
}

impl From<Quat> for Mat4 {
    #[inline(never)]
    fn from(q: Quat) -> Self {
        let xx = q.x * q.x;
        let yy = q.y * q.y;
        let zz = q.z * q.z;
        let xz = q.x * q.z;
        let xy = q.x * q.y;
        let yz = q.y * q.z;
        let wx = q.w * q.x;
        let wy = q.w * q.y;
        let wz = q.w * q.z;

        let a = [
            1.0 - 2.0 * (yy + zz), //br
            2.0 * (xy + wz),       //br
            2.0 * (xz - wy),       //br
            0.0,
        ];
        let b = [
            2.0 * (xy - wz),       //br
            1.0 - 2.0 * (xx + zz), //br
            2.0 * (yz + wx),       //br
            0.0,
        ];
        let c = [
            2.0 * (xz + wy),       //br
            2.0 * (yz - wx),       //br
            1.0 - 2.0 * (xx + yy), //br
            0.0,
        ];
        let d = [0.0, 0.0, 0.0, 1.0];
        Mat4::from([a, b, c, d])
    }
}

#[cfg(test)]
pub mod tests {
    use ::float_cmp::ApproxEq;
    use ::std::default::Default;
    use ::std::{assert_eq, assert_ne, panic};
    use ::wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    impl ::std::cmp::PartialEq for Vec3 {
        fn eq(&self, other: &Self) -> bool {
            let m = ::float_cmp::F32Margin::default();
            self.x.approx_eq(other.x, m)
                && self.y.approx_eq(other.y, m)
                && self.z.approx_eq(other.z, m)
        }
    }

    impl ::std::cmp::PartialEq for Mat4 {
        fn eq(&self, other: &Mat4) -> bool {
            #[allow(unsafe_code)]
            unsafe {
                self.buf == other.buf
            }
        }
    }

    #[wasm_bindgen_test]
    fn test_vec3_approx_eq() {
        let v1 = Vec3::with(1.0, 0.0, 0.0);
        let v2 = Vec3::with(0.0, 0.0, -1.0);
        assert_ne!(v1, v2);
    }

    #[wasm_bindgen_test]
    fn test_vec4_dot() {
        let v1 = Vec4::with(1.0, 5.0, 9.0, 13.0);
        let v2 = Vec4::with(2.0, 2.0, 2.0, 2.0);
        assert_eq!(v1.dot(v2), (1 * 2 + 5 * 2 + 9 * 2 + 13 * 2) as f32);
    }

    #[wasm_bindgen_test]
    fn test_mat4_row() {
        let m = Mat4::from([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        assert_eq!(m.row(0), [1.0, 5.0, 9.0, 13.0]);
        assert_eq!(m.row(1), [2.0, 6.0, 10.0, 14.0]);
        assert_eq!(m.row(2), [3.0, 7.0, 11.0, 15.0]);
        assert_eq!(m.row(3), [4.0, 8.0, 12.0, 16.0]);
    }

    #[wasm_bindgen_test]
    fn test_mat4_union() {
        let m1 = Mat4::from([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        let m2 = Mat4::from([
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ]);
        assert_eq!(m1, m2);
    }

    #[wasm_bindgen_test]
    fn test_mat4_transpose() {
        let mut m1 = Mat4::from([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        let m2 = Mat4::from([
            1.0, 5.0, 9.0, 13.0, //br
            2.0, 6.0, 10.0, 14.0, //br
            3.0, 7.0, 11.0, 15.0, //br
            4.0, 8.0, 12.0, 16.0, //br
        ]);
        m1.transpose();
        assert_eq!(m1, m2);
    }

    #[wasm_bindgen_test]
    fn test_mat4_mul() {
        let m1 = Mat4::from([
            1.0, -2.0, -2.0, -2.0, //br
            2.0, 1.0, -2.0, -2.0, //br
            2.0, 2.0, 1.0, -2.0, //br
            2.0, 2.0, 2.0, 1.0, //br
        ]);
        let m2 = Mat4::from([
            1.0, 2.0, 3.0, 4.0, //br
            5.0, 6.0, 7.0, 8.0, //br
            9.0, 10.0, 11.0, 12.0, //br
            13.0, 14.0, 15.0, 16.0, //br
        ]);
        let m3 = Mat4::from([
            19.0, 14.0, 5.0, -8.0, //br
            47.0, 26.0, 1.0, -28.0, //br
            75.0, 38.0, -3.0, -48.0, //br
            103.0, 50.0, -7.0, -68.0, //br
        ]);
        assert_eq!(m1 * m2, m3);
    }

    #[wasm_bindgen_test]
    fn test_vec3_unit_cartesian_to_polar() {
        assert_eq!(
            Vec3::with(0.0, 0.0, 0.0).to_polar(),
            Vec3::with(0.0, 0.0, 0.0)
        );
        assert_eq!(
            Vec3::with(1.0, 0.0, 0.0).to_polar(),
            Vec3::with(1.0, 90.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, 1.0, 0.0).to_polar(),
            Vec3::with(1.0, 0.0, 90.0)
        );
        assert_eq!(
            Vec3::with(0.0, 0.0, 1.0).to_polar(),
            Vec3::with(1.0, 0.0, 0.0)
        );
        assert_eq!(
            Vec3::with(-1.0, 0.0, 0.0).to_polar(),
            Vec3::with(1.0, -90.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, -1.0, 0.0).to_polar(),
            Vec3::with(1.0, 0.0, -90.0)
        );
        assert_eq!(
            Vec3::with(0.0, 0.0, -1.0).to_polar(),
            Vec3::with(1.0, 180.0, 0.0)
        );

        let sqrt2 = 2.0_f32.sqrt();
        let sqrt3 = 3.0_f32.sqrt();

        assert_eq!(
            Vec3::with(1.0, 0.0, 1.0).to_polar(),
            Vec3::with(sqrt2, 45.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, 1.0, 1.0).to_polar(),
            Vec3::with(sqrt2, 0.0, 45.0)
        );
        assert_eq!(
            Vec3::with(1.0, 1.0, 0.0).to_polar(),
            Vec3::with(sqrt2, 90.0, 45.0)
        );
        assert_eq!(
            Vec3::with(1.0, 1.0, 1.0).to_polar(),
            Vec3::with(sqrt3, 45.0, 35.26439)
        );
        assert_eq!(
            Vec3::with(-1.0, 0.0, 1.0).to_polar(),
            Vec3::with(sqrt2, -45.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, -1.0, 1.0).to_polar(),
            Vec3::with(sqrt2, 0.0, -45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, -1.0, 0.0).to_polar(),
            Vec3::with(sqrt2, -90.0, -45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, -1.0, 1.0).to_polar(),
            Vec3::with(sqrt3, -45.0, -35.26439)
        );
        assert_eq!(
            Vec3::with(-1.0, 0.0, -1.0).to_polar(),
            Vec3::with(sqrt2, -135.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, 1.0, -1.0).to_polar(),
            Vec3::with(sqrt2, 180.0, 45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, 1.0, 0.0).to_polar(),
            Vec3::with(sqrt2, -90.0, 45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, 1.0, -1.0).to_polar(),
            Vec3::with(sqrt3, -135.0, 35.26439)
        );
        assert_eq!(
            Vec3::with(1.0, 0.0, -1.0).to_polar(),
            Vec3::with(sqrt2, 135.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, -1.0, -1.0).to_polar(),
            Vec3::with(sqrt2, 180.0, -45.0)
        );
        assert_eq!(
            Vec3::with(1.0, -1.0, 0.0).to_polar(),
            Vec3::with(sqrt2, 90.0, -45.0)
        );
        assert_eq!(
            Vec3::with(1.0, -1.0, -1.0).to_polar(),
            Vec3::with(sqrt3, 135.0, -35.26439)
        );
        assert_eq!(
            Vec3::with(-1.0, 0.0, -1.0).to_polar(),
            Vec3::with(sqrt2, -135.0, 0.0)
        );
        assert_eq!(
            Vec3::with(0.0, -1.0, -1.0).to_polar(),
            Vec3::with(sqrt2, 180.0, -45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, -1.0, 0.0).to_polar(),
            Vec3::with(sqrt2, -90.0, -45.0)
        );
        assert_eq!(
            Vec3::with(-1.0, -1.0, -1.0).to_polar(),
            Vec3::with(sqrt3, -135.0, -35.26439)
        );
    }

    #[wasm_bindgen_test]
    fn test_vec3_unit_polar_to_cartesian() {
        assert_eq!(
            Vec3::with(1.0, 90.0, 0.0).to_cartesian(),
            Vec3::with(1.0, 0.0, 0.0)
        );
        assert_eq!(
            Vec3::with(1.0, 0.0, 90.0).to_cartesian(),
            Vec3::with(0.0, 1.0, 0.0)
        );
        assert_eq!(
            Vec3::with(1.0, 0.0, 0.0).to_cartesian(),
            Vec3::with(0.0, 0.0, 1.0)
        );
        assert_eq!(
            Vec3::with(1.0, -90.0, 0.0).to_cartesian(),
            Vec3::with(-1.0, 0.0, 0.0)
        );
        assert_eq!(
            Vec3::with(1.0, 0.0, -90.0).to_cartesian(),
            Vec3::with(0.0, -1.0, 0.0)
        );
        assert_eq!(
            Vec3::with(1.0, 180.0, 0.0).to_cartesian(),
            Vec3::with(0.0, 0.0, -1.0)
        );

        let sqrt2 = 2.0_f32.sqrt();

        assert_eq!(
            Vec3::with(sqrt2, 45.0, 0.0).to_cartesian(),
            Vec3::with(1.0, 0.0, 1.0)
        );
        assert_eq!(
            Vec3::with(sqrt2, 0.0, 45.0).to_cartesian(),
            Vec3::with(0.0, 1.0, 1.0)
        );
        assert_eq!(
            Vec3::with(sqrt2, 90.0, 45.0).to_cartesian(),
            Vec3::with(1.0, 1.0, 0.0)
        );
        assert_eq!(
            Vec3::with(sqrt2, -90.0, 45.0).to_cartesian(),
            Vec3::with(-1.0, 1.0, 0.0)
        );
        assert_eq!(
            Vec3::with(sqrt2, 123.4, 90.0).to_cartesian(),
            Vec3::with(0.0, sqrt2, 0.0)
        );
    }

    impl Vec3 {
        fn test_rotate(&self, angles: [f32; 3]) -> Vec3 {
            let m: Mat4 = Quat::rotation(angles.into()).into();
            (m * Vec4::from((*self, 1.0))).xyz()
        }
    }

    #[wasm_bindgen_test]
    fn test_quaternion_rotation() {
        let actual = Vec3::with(1.0, 0.0, 0.0).test_rotate([90.0, 0.0, 0.0]);
        assert_eq!(actual, Vec3::with(1.0, 0.0, 0.0));
        let actual = Vec3::with(1.0, 0.0, 0.0).test_rotate([0.0, 90.0, 0.0]);
        assert_eq!(actual, Vec3::with(0.0, 0.0, -1.0));
        let actual = Vec3::with(1.0, 0.0, 0.0).test_rotate([0.0, 0.0, 90.0]);
        assert_eq!(actual, Vec3::with(0.0, 1.0, 0.0));

        let actual = Vec3::with(0.0, 1.0, 0.0).test_rotate([90.0, 0.0, 0.0]);
        assert_eq!(actual, Vec3::with(0.0, 0.0, 1.0));
        let actual = Vec3::with(0.0, 1.0, 0.0).test_rotate([0.0, 90.0, 0.0]);
        assert_eq!(actual, Vec3::with(0.0, 1.0, 0.0));
        let actual = Vec3::with(0.0, 1.0, 0.0).test_rotate([0.0, 0.0, 90.0]);
        assert_eq!(actual, Vec3::with(-1.0, 0.0, 0.0));

        let actual = Vec3::with(0.0, 0.0, 1.0).test_rotate([90.0, 0.0, 0.0]);
        assert_eq!(actual, Vec3::with(0.0, -1.0, 0.0));
        let actual = Vec3::with(0.0, 0.0, 1.0).test_rotate([0.0, 90.0, 0.0]);
        assert_eq!(actual, Vec3::with(1.0, 0.0, 0.0));
        let actual = Vec3::with(0.0, 0.0, 1.0).test_rotate([0.0, 0.0, 90.0]);
        assert_eq!(actual, Vec3::with(0.0, 0.0, 1.0));
    }

    #[wasm_bindgen_test]
    fn test_alignments() {
        assert_eq!(::std::mem::align_of::<Vec3>(), 16);
        assert_eq!(::std::mem::align_of::<Vec4>(), 16);
        assert_eq!(::std::mem::align_of::<Mat4>(), 16);
    }
}
