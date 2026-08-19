struct Globals {
    _pad: vec4<f32>,
};
var<uniform> globals: Globals;

struct Locals {
    pos_size: vec4<f32>,
    uv_rect: vec4<f32>,
    tint: vec4<f32>,
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
    return o;
}

@fragment
fn fs_main(v: VSOut) -> @location(0) vec4<f32> {
    let texel = textureSampleLevel(sprite_texture, sprite_sampler, v.uv, 0.0);
    return texel * v.tint;
}
