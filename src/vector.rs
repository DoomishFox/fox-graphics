#[repr(C)]
#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

pub struct Mat4<T> {
    pub x: Vector4<T>,
    pub y: Vector4<T>,
    pub z: Vector4<T>,
    pub w: Vector4<T>,
}