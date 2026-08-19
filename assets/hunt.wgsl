struct Globals {
    origin_zoom: vec4<f32>,
    screen: vec4<f32>,
    light_dir: vec4<f32>,
};
var<uniform> globals: Globals;

struct Locals {
    world: vec4<f32>,
    color: vec4<f32>,
};
var<uniform> locals: Locals;

struct Vertex {
    pos: vec3<f32>,
    normal: vec3<f32>,
};

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(v: Vertex) -> VSOut {
    let p = vec3<f32>(
        v.pos.x * locals.world.w,
        v.pos.y * locals.color.w,
        v.pos.z * locals.world.w,
    ) + locals.world.xyz;
    let iso_x = (p.x - p.z) * 0.8660254;
    let iso_y = (p.x + p.z) * 0.5 - p.y;
    let sx = globals.origin_zoom.x + iso_x * globals.origin_zoom.z;
    let sy = globals.origin_zoom.y + iso_y * globals.origin_zoom.z;
    let ndc_x = sx / max(globals.screen.x, 1.0) * 2.0 - 1.0;
    let ndc_y = 1.0 - sy / max(globals.screen.y, 1.0) * 2.0;
    let depth = clamp(0.55 - (p.x + p.z) * 0.003 - p.y * 0.02, 0.01, 0.99);
    let n = normalize(v.normal);
    let ndl = max(dot(n, globals.light_dir.xyz), 0.0);
    var o: VSOut;
    o.clip = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    o.color = vec4<f32>(locals.color.xyz * (0.32 + 0.68 * ndl), 1.0);
    return o;
}

@fragment
fn fs_main(v: VSOut) -> @location(0) vec4<f32> {
    return v.color;
}
