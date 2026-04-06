use cgmath::InnerSpace;
use three_d_asset::{Vec3, Vec4};

/// Represents the clip plane of a geometry.
#[derive(Debug, Clone, Copy)]
pub struct ClipPlane {
    nomal: Vec3,
    distance: f32,
}

impl ClipPlane {
    /// Creates a new clip plane with the given nomal and point.
    /// The distance is calculated with ```-nomal.dot(point)```
    pub fn new(point: Vec3, nomal: Vec3) -> Self {
        let distance = -nomal.dot(point);
        Self { nomal, distance }
    }

    /// Creates a new clip plane with the given nomal and distance.
    pub fn new_with_distance(nomal: Vec3, distance: f32) -> Self {
        Self { nomal, distance }
    }

    /// Sets the clip plane with the given nomal and point.
    pub fn set(&mut self, point: Vec3, nomal: Vec3) {
        self.distance = -nomal.dot(point);
        self.nomal = nomal;
    }

    /// Returns the clip plane as a 4D vector for use as a uniform in a shader.
    pub fn as_vec4(&self) -> Vec4 {
        self.nomal.extend(self.distance)
    }
}
