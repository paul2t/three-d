use crate::*;
use three_d_asset::Positions;

///
/// Base implementation of [Lines]
///
pub(crate) struct BaseLines {
    pub(crate) positions: VertexBuffer<Vec3>,
    pub(crate) colors: Option<VertexBuffer<Vec2>>,
    pub(crate) indices: IndexBuffer,
}

impl BaseLines {
    pub(crate) fn new(context: &Context, positions: &Positions) -> Self {
        Self {
            positions: VertexBuffer::new_with_data(context, &positions.to_f32()),
            colors: None,
            indices: IndexBuffer::None,
        }
    }

    pub(crate) fn draw(&self, program: &Program, render_states: RenderStates, viewer: &dyn Viewer) {
        #[cfg(debug_assertions)]
        assert!(self.positions.vertex_count() % 2 == 0);
        self.use_attributes(program);

        match &self.indices {
            IndexBuffer::None => program.draw_arrays(
                render_states,
                viewer.viewport(),
                self.positions.vertex_count(),
            ),
            IndexBuffer::U8(element_buffer) => {
                program.draw_elements(render_states, viewer.viewport(), element_buffer)
            }
            IndexBuffer::U16(element_buffer) => {
                program.draw_elements(render_states, viewer.viewport(), element_buffer)
            }
            IndexBuffer::U32(element_buffer) => {
                program.draw_elements(render_states, viewer.viewport(), element_buffer)
            }
        }
    }

    fn use_attributes(&self, program: &Program) {
        program.use_vertex_attribute("position", &self.positions);

        if program.requires_attribute("color") {
            if let Some(colors) = &self.colors {
                program.use_vertex_attribute("color", colors);
            }
        }
    }

    pub(crate) fn vertex_shader_source(&self) -> String {
        format!(
            "{}{}",
            if self.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            include_str!("shaders/lines.vert"),
        )
    }
}
