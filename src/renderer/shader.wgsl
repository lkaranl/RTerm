// Shader para renderização de células do terminal
// Otimizado para legibilidade com Gamma Correction

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
    // 1. Amostra do glyph (alpha map)
    var alpha = 0.0;
    if (in.tex_coords.x > 0.0 || in.tex_coords.y > 0.0) {
        alpha = textureSample(t_glyph, s_glyph, in.tex_coords).a;
        
        // Sharpening leve / Gamma correction para texto
        // Isso faz o texto parecer mais "bold" e nítido
        alpha = pow(alpha, 1.0 / 1.4); 
    }

    // 2. Mistura background e foreground
    // Mix linear: bg * (1 - alpha) + fg * alpha
    var color = mix(in.bg_color, in.fg_color, alpha);
    
    return color;
}
