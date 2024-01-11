use crate::core::*;
use crate::renderer::geometry::BaseLines;
use crate::renderer::*;

///
/// A [Geometry] representing lines.
///
pub struct Lines {
    base_line: BaseLines,
    context: Context,
    aabb: AxisAlignedBoundingBox,
    transformation: Mat4,
    current_transformation: Mat4,
    animation: Option<Box<dyn Fn(f32) -> Mat4 + Send + Sync>>,
}

impl Lines {
    ///
    /// Creates a new line mesh from the given [Positions].
    /// All data in the [Positions] is transfered to the GPU, so make sure to remove all unnecessary data from the [Positions] before calling this method.
    ///
    pub fn new(context: &Context, positions: &Positions) -> Self {
        let aabb = positions.compute_aabb();
        Self {
            context: context.clone(),
            base_line: BaseLines::new(context, positions),
            aabb,
            transformation: Mat4::identity(),
            current_transformation: Mat4::identity(),
            animation: None,
        }
    }

    ///
    /// Returns the local to world transformation applied to this mesh.
    ///
    pub fn transformation(&self) -> Mat4 {
        self.transformation
    }

    ///
    /// Set the local to world transformation applied to this mesh.
    /// If any animation method is set using [Self::set_animation], the transformation from that method is applied before this transformation.
    ///
    pub fn set_transformation(&mut self, transformation: Mat4) {
        self.transformation = transformation;
        self.current_transformation = transformation;
    }

    ///
    /// Specifies a function which takes a time parameter as input and returns a transformation that should be applied to this mesh at the given time.
    /// To actually animate this mesh, call [Geometry::animate] at each frame which in turn evaluates the animation function defined by this method.
    /// This transformation is applied first, then the local to world transformation defined by [Self::set_transformation].
    ///
    pub fn set_animation(&mut self, animation: impl Fn(f32) -> Mat4 + Send + Sync + 'static) {
        self.animation = Some(Box::new(animation));
    }
}

impl<'a> IntoIterator for &'a Lines {
    type Item = &'a dyn Geometry;
    type IntoIter = std::iter::Once<&'a dyn Geometry>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl Geometry for Lines {
    fn aabb(&self) -> AxisAlignedBoundingBox {
        let mut aabb = self.aabb;
        aabb.transform(&self.current_transformation);
        aabb
    }

    fn animate(&mut self, time: f32) {
        if let Some(animation) = &self.animation {
            self.current_transformation = self.transformation * animation(time);
        }
    }

    fn draw(
        &self,
        camera: &Camera,
        program: &Program,
        render_states: RenderStates,
        attributes: FragmentAttributes,
    ) {
        program.use_uniform("viewProjection", camera.projection() * camera.view());
        program.use_uniform("modelMatrix", self.current_transformation);

        self.base_line
            .draw(program, render_states, camera, attributes);
    }

    fn vertex_shader_source(&self, required_attributes: FragmentAttributes) -> String {
        format!(
            "{}{}",
            if required_attributes.color && self.base_line.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            include_str!("shaders/lines.vert"),
        )
    }

    fn vertex_type(&self) -> u32 {
        crate::context::LINES
    }

    fn id(&self, required_attributes: FragmentAttributes) -> u16 {
        let mut id = 0b1u16 << 15 | 0b1u16 << 8;
        if required_attributes.normal {
            id |= 0b1u16;
        }
        if required_attributes.tangents {
            id |= 0b1u16 << 1;
        }
        if required_attributes.uv {
            id |= 0b1u16 << 2;
        }
        if required_attributes.color && self.base_line.colors.is_some() {
            id |= 0b1u16 << 3;
        }
        id
    }

    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        render_with_material(&self.context, camera, &self, material, lights);
    }

    fn render_with_effect(
        &self,
        material: &dyn Effect,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        render_with_effect(
            &self.context,
            camera,
            self,
            material,
            lights,
            color_texture,
            depth_texture,
        )
    }
}
