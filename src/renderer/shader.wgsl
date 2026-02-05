// Shader para renderização de células do terminal
// Otimizado para Apple Silicon

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) fg_color: vec4<f32>,
    @location(3) bg_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) fg_color: vec4<f32>,
    @location(2) bg_color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.tex_coords = in.tex_coords;
    out.fg_color = in.fg_color;
    out.bg_color = in.bg_color;
    return out;
}

@group(0) @binding(0)
var t_glyph: texture_2d<f32>;
@group(0) @binding(1)
var s_glyph: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Background
    var color = in.bg_color;
    
    // Se houver UV válido, aplica o glyph
    if (in.tex_coords.x > 0.0 || in.tex_coords.y > 0.0) {
        let glyph = textureSample(t_glyph, s_glyph, in.tex_coords);
        // Blend foreground sobre background usando alpha do glyph
        color = mix(in.bg_color, in.fg_color, glyph.a);
    }
    
    return color;
}
