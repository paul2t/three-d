use crate::renderer::*;

///
/// Resolves the Order-Independent Transparency (OIT) effect.
///
#[derive(Clone, Debug)]
pub struct OitResolveEffect {
    /// Defines which type of blending to use when writing the copied color to the render target.
    pub blend: Blend,
}

impl Default for OitResolveEffect {
    fn default() -> Self {
        Self {
            blend: Blend::Enabled {
                source_rgb_multiplier: BlendMultiplierType::OneMinusSrcAlpha,
                source_alpha_multiplier: BlendMultiplierType::OneMinusSrcAlpha,
                destination_rgb_multiplier: BlendMultiplierType::SrcAlpha,
                destination_alpha_multiplier: BlendMultiplierType::SrcAlpha,
                rgb_equation: BlendEquationType::Add,
                alpha_equation: BlendEquationType::Add,
            },
        }
    }
}

impl Effect for OitResolveEffect {
    fn fragment_shader_source(
        &self,
        _lights: &[&dyn crate::Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) -> String {
        format!(
            "{color}\n{depth}\n{tone_mapping}\n{color_mapping}\n{shader}",
            color = color_texture
                .map(|t| t.fragment_shader_source())
                .unwrap_or("".to_string()),
            depth = depth_texture
                .map(|t| t.fragment_shader_source())
                .unwrap_or("".to_string()),
            tone_mapping = ToneMapping::fragment_shader_source(),
            color_mapping = ColorMapping::fragment_shader_source(),
            shader = include_str!("shaders/oit_resolve_effect.frag"),
        )
    }

    fn id(
        &self,
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) -> EffectMaterialId {
        EffectMaterialId::OitResolveEffect(color_texture, depth_texture)
    }

    fn use_uniforms(
        &self,
        program: &Program,
        viewer: &dyn Viewer,
        _lights: &[&dyn crate::Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        if let Some(color_texture) = color_texture {
            color_texture.use_uniforms(program);
        }
        if let Some(depth_texture) = depth_texture {
            depth_texture.use_uniforms(program);
        }
        viewer.tone_mapping().use_uniforms(program);
        viewer.color_mapping().use_uniforms(program);
    }

    fn render_states(&self) -> RenderStates {
        RenderStates {
            depth_test: DepthTest::Always,
            cull: Cull::None,
            write_mask: WriteMask::COLOR,
            blend: self.blend,
            line_width: 1.0,
        }
    }
}
