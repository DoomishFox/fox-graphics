struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) edge: f32,
    @location(2) half_width: f32,
};
@vertex fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>,
    @location(2) edge: f32, @location(3) half_width: f32) -> Out {
    var out: Out;
    out.position = vec4(position, 0.0, 1.0);
    out.color = color;
    out.edge = edge;
    out.half_width = half_width;
    return out;
}
@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let aa = max(fwidth(in.edge), 0.001);
    let coverage = 1.0 - smoothstep(in.half_width - aa * 0.5, in.half_width + aa * 0.5, abs(in.edge));
    return vec4(in.color.rgb, in.color.a * coverage);
}
