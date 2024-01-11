use crate::*;
use three_d_asset::Positions;

///
/// Base implementation of [Lines]
///
pub(crate) struct BaseLines {
    pub(crate) positions: VertexBuffer,
    pub(crate) colors: Option<VertexBuffer>,
    pub(crate) indices: Option<ElementBuffer>,
}

impl BaseLines {
    pub(crate) fn new(context: &Context, positions: &Positions) -> Self {
        Self {
            positions: VertexBuffer::new_with_data(context, &positions.to_f32()),
            colors: None,
            indices: None,
        }
    }

    pub(crate) fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        camera: &Camera,
        attributes: FragmentAttributes,
    ) {
        self.use_attributes(program, attributes);
        if let Some(index_buffer) = &self.indices {
            program.draw_elements(render_states, camera.viewport(), index_buffer)
        } else {
            program.draw_arrays(
                render_states,
                camera.viewport(),
                self.positions.vertex_count(),
            )
        }
    }

    fn use_attributes(&self, program: &Program, attributes: FragmentAttributes) {
        program.use_vertex_attribute("position", &self.positions);

        if attributes.color {
            if let Some(colors) = &self.colors {
                program.use_vertex_attribute("color", colors);
            }
        }
    }
}
