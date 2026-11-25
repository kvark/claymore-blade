struct Globals {
    _pad: vec4<f32>,
};
var<uniform> globals: Globals;

struct Locals {
    pos_size: vec4<f32>,
    uv_rect: vec4<f32>,
    tint: vec4<f32>,
    // C2 screentone: dot pitch (px), strength (0 = plain sprite), edge band
    // (in rect heights), rect aspect (w / h in px).
    tone: vec4<f32>,
};
var<uniform> locals: Locals;

struct Vertex {
    pos: vec2<f32>,
};

var sprite_texture: texture_2d<f32>;
var sprite_sampler: sampler;

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
    @location(2) tone: vec4<f32>,
};

@vertex
fn vs_main(v: Vertex) -> VSOut {
    let p = vec2<f32>(
        locals.pos_size.x + v.pos.x * locals.pos_size.z,
        locals.pos_size.y + v.pos.y * locals.pos_size.w,
    );
    var o: VSOut;
    // Pin globals to the vertex stage so WebGL2 does not see a nameless block in both shaders.
    o.clip = vec4<f32>(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, globals._pad.x * 0.0, 1.0);
    o.uv = locals.uv_rect.xy + v.pos * locals.uv_rect.zw;
    o.tint = locals.tint;
    o.tone = locals.tone;
    return o;
}

@fragment
fn fs_main(v: VSOut) -> @location(0) vec4<f32> {
    let texel = textureSampleLevel(sprite_texture, sprite_sampler, v.uv, 0.0);
    // Print tone: round ink dots on a 45-degree grid fixed to screen pixels
    // (no scaling, so no moire); dot area follows the vignette coverage.
    // Derivative taken before the branch (uniform control flow).
    let g = vec2<f32>(v.clip.x + v.clip.y, v.clip.x - v.clip.y) * 0.70710678 / max(v.tone.x, 1.0);
    let d = length(fract(g) - vec2<f32>(0.5));
    let aa = max(fwidth(d), 0.03);
    if (v.tone.y <= 0.0) {
        return texel * v.tint;
    }
    let e = min(min(v.uv.x, 1.0 - v.uv.x) * v.tone.w, min(v.uv.y, 1.0 - v.uv.y));
    let cov = (1.0 - smoothstep(0.0, v.tone.z, e)) * v.tone.y;
    let r = 0.56 * sqrt(cov);
    let ink = 1.0 - smoothstep(r - aa, r + aa, d);
    return vec4<f32>(v.tint.rgb, v.tint.a * ink);
}
