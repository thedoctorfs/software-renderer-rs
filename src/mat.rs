use std::cmp;
use std::ops::{Add, Mul};
use crate::vec::Vec4;

pub struct Mat4x4<T>(T, T, T, T,
                        T, T, T, T,
                        T, T, T, T,
                        T, T, T, T);

/*impl<T: Mul<Output = T> + Add<Output=T> + Copy> Mul for Mat4x4<T> {
    type Output = Vec4<T>;
    fn mul(self, rhs: Vec4<T>) -> Vec4<T> {
        Vec4 {
            x: self.0 * rhs.x + self.1 * rhs.y + self.2 * rhs.z + self.3 * rhs.w,
            y: self.4 * rhs.x + self.5 * rhs.y + self.6 * rhs.z + self.7 * rhs.w,
            z: self.8 * rhs.x + self.9 * rhs.y + self.10 * rhs.z + self.11 * rhs.w,
            w: self.12 * rhs.x + self.13 * rhs.y + self.14 * rhs.z + self.15 * rhs.w,
        }
    }
}*/