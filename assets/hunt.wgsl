struct Globals {
    origin_zoom: vec4<f32>,
    screen: vec4<f32>,
    light_dir: vec4<f32>,
    lamp0: vec4<f32>,
    lamp0c: vec4<f32>,
    lamp1: vec4<f32>,
    lamp1c: vec4<f32>,
};
var<uniform> globals: Globals;

struct Locals {
    world: vec4<f32>,
    color: vec4<f32>,
    pose: vec4<f32>,
    joints: array<mat3x4<f32>, 8>,
};
var<uniform> locals: Locals;

struct Vertex {
    pos: vec3<f32>,
    normal: vec3<f32>,
    joints: u32,
    weights: u32,
};

struct VSOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn unpack_joints(raw: u32) -> vec4<u32> {
    return (vec4<u32>(raw) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(0xFFu);
}

fn apply_affine(m: mat3x4<f32>, p: vec3<f32>) -> vec3<f32> {
    return vec4<f32>(p, 1.0) * m;
}

fn skin_blend(j: u32, w: u32) -> mat3x4<f32> {
    let joints = unpack_joints(j);
    let weights = vec4<f32>(f32(w & 0xFFu), f32((w >> 8u) & 0xFFu), f32((w >> 16u) & 0xFFu), f32((w >> 24u) & 0xFFu)) / 255.0;
    return locals.joints[joints.x] * weights.x
        + locals.joints[joints.y] * weights.y
        + locals.joints[joints.z] * weights.z
        + locals.joints[joints.w] * weights.w;
}

fn lamp(p: vec3<f32>, n: vec3<f32>, pos_i: vec4<f32>, col_r: vec4<f32>) -> vec3<f32> {
    let to_l = pos_i.xyz - p;
    let dist = length(to_l);
    let radius = max(col_r.w, 1.0);
    let att = pos_i.w / (1.0 + (dist * dist) / (radius * radius));
    let ndl = max(dot(n, normalize(to_l + vec3<f32>(0.0, 0.001, 0.0))), 0.0);
    return col_r.xyz * att * (0.35 + 0.65 * ndl);
}

@vertex
fn vs_main(v: Vertex) -> VSOut {
    let skin = skin_blend(v.joints, v.weights);
    let linear = transpose(mat3x3<f32>(skin[0].xyz, skin[1].xyz, skin[2].xyz));
    var lp = apply_affine(skin, v.pos);
    let c = locals.pose.x;
    let s = locals.pose.y;
    let rx = lp.x * c - lp.z * s;
    let rz = lp.x * s + lp.z * c;
    lp = vec3<f32>(rx, lp.y + locals.pose.z * rx, rz);
    var n0 = linear * v.normal;
    n0 = vec3<f32>(n0.x * c - n0.z * s, n0.y, n0.x * s + n0.z * c);
    let p = vec3<f32>(
        lp.x * locals.world.w,
        lp.y * locals.color.w,
        lp.z * locals.world.w,
    ) + locals.world.xyz;
    let iso_x = (p.x - p.z) * 0.8660254;
    let iso_y = (p.x + p.z) * 0.5 - p.y;
    let sx = globals.origin_zoom.x + iso_x * globals.origin_zoom.z;
    let sy = globals.origin_zoom.y + iso_y * globals.origin_zoom.z;
    let ndc_x = sx / max(globals.screen.x, 1.0) * 2.0 - 1.0;
    let ndc_y = 1.0 - sy / max(globals.screen.y, 1.0) * 2.0;
    let depth = clamp(0.55 - (p.x + p.z) * 0.003 - p.y * 0.02, 0.01, 0.99);
    let n = normalize(n0);
    let sun = max(dot(n, normalize(globals.light_dir.xyz)), 0.0);
    var lit = locals.color.xyz * (0.16 + 0.62 * sun);
    lit += lamp(p, n, globals.lamp0, globals.lamp0c);
    lit += lamp(p, n, globals.lamp1, globals.lamp1c);
    lit += locals.color.xyz * locals.pose.w;
    var o: VSOut;
    o.clip = vec4<f32>(ndc_x, ndc_y, depth, 1.0);
    o.color = vec4<f32>(lit, 1.0);
    return o;
}

@fragment
fn fs_main(v: VSOut) -> @location(0) vec4<f32> {
    return v.color;
}
