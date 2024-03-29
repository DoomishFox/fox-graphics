struct TextPipeline {
    char_vertex_buffer: wgpu::Buffer,
    wgpu_pipeline: wgpu::RenderPipeline,
    glyph_bind_group: wgpu::BindGroup,
}

impl TextPipeline {
    pub fn new() -> Self {
        
    }
}

struct GlyphRun {
    // textline
    // - glyph buffer
    // - metadata
    // text wgpu buffer
    // text wgpu bind group
}