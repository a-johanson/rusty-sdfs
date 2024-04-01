use crate::vector::Vec3;
use crate::sdf::SdfOutput;

pub trait Scene {
    fn eval(&self, p: &Vec3) -> SdfOutput;
}
