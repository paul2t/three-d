use crate::renderer::*;

///
/// A wireframe geometry that can be constructed from a [CpuMesh].
/// Consisting of [Lines] and a [ColorMaterial], with a line `width`.
///
pub struct Wireframe {
    lines: Gm<Lines, ColorMaterial>,
}

impl Wireframe {
    ///
    /// Constructs a new [Wireframe] object from the given [CpuMesh].
    /// The default `color` is black. The default line `width` is 1.0.
    ///
    pub fn new(context: &Context, model: &CpuMesh) -> Self {
        Self::new_with_color_and_line_width(context, model, Srgba::BLACK, 1.0)
    }

    ///
    /// Constructs a new [Wireframe] object from the given [CpuMesh] with a specified `color`.
    /// The default line `width` is 1.0.
    ///
    pub fn new_with_color(context: &Context, model: &CpuMesh, color: Srgba) -> Self {
        Self::new_with_color_and_line_width(context, model, color, 1.0)
    }

    ///
    /// Constructs a new [Wireframe] object from the given [CpuMesh] with a specified color and a specified line `width`.
    ///
    pub fn new_with_color_and_line_width(
        context: &Context,
        model: &CpuMesh,
        color: Srgba,
        line_width: f32,
    ) -> Self {
        let lines = Self::get_wireframe_lines(model);
        let positions = Positions::F32(lines);
        let lines = Lines::new(context, &positions);
        Self::new_with_lines(lines, color, line_width)
    }

    ///
    /// Constructs a new [Wireframe] object from the given [Lines], `color` and line `width`.
    ///
    pub fn new_with_lines(lines: Lines, color: Srgba, line_width: f32) -> Self {
        let material = ColorMaterial {
            color,
            render_states: RenderStates {
                depth_test: DepthTest::LessOrEqual,
                line_width,
                ..Default::default()
            },
            ..Default::default()
        };
        Wireframe {
            lines: Gm::new(lines, material),
        }
    }

    ///
    /// Set the `color` of the lines.
    ///
    pub fn set_color(&mut self, color: Srgba) {
        self.lines.material.color = color;
    }

    ///
    /// Set the `width` of the lines.
    ///
    pub fn set_line_width(&mut self, line_width: f32) {
        self.lines.material.render_states.line_width = line_width;
    }

    ///
    /// Helper function to get the lines from a [CpuMesh].
    /// To easily create an array of positions: [Positions::F32]
    /// ```
    /// let lines = Wireframe::get_wireframe_lines(model);
    /// let positions = Positions::F32(lines);
    /// let lines = Lines::new(context, &positions);
    /// ```
    ///
    pub fn get_wireframe_lines(model: &CpuMesh) -> Vec<Vec3> {
        let mut lines = Vec::new();
        let indices = model.indices.to_u32().unwrap();
        let positions = model.positions.to_f32();
        for f in 0..indices.len() / 3 {
            let i1 = indices[3 * f] as usize;
            let i2 = indices[3 * f + 1] as usize;
            let i3 = indices[3 * f + 2] as usize;

            if i1 < i2 {
                lines.push(positions[i1]);
                lines.push(positions[i2]);
            }
            if i2 < i3 {
                lines.push(positions[i2]);
                lines.push(positions[i3]);
            }
            if i3 < i1 {
                lines.push(positions[i3]);
                lines.push(positions[i1]);
            }
        }
        lines
    }
}

impl<'a> IntoIterator for &'a Wireframe {
    type Item = &'a dyn Object;
    type IntoIter = std::iter::Once<&'a dyn Object>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl Object for Wireframe {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.lines.render(camera, lights);
    }

    fn material_type(&self) -> MaterialType {
        self.lines.material_type()
    }
}

use std::ops::Deref;
impl Deref for Wireframe {
    type Target = Lines;
    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

impl std::ops::DerefMut for Wireframe {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.lines
    }
}

impl Geometry for Wireframe {
    impl_geometry_body!(deref);

    fn animate(&mut self, time: f32) {
        self.lines.animate(time)
    }
}
