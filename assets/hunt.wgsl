struct Globals {
    origin_zoom: vec4<f32>,
    screen: vec4<f32>,
    depth_origin_scale: vec4<f32>,
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
    rim: vec4<f32>,
    mat: vec4<f32>,
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
    // Board-relative position in tile units (xyz) + time (w), for the
    // per-pixel terrain / ground materials. Varyings only: fs_main reads no
    // uniforms, so WebGL2 never sees a block shared by both stages.
    @location(1) rel: vec4<f32>,
    @location(2) normal: vec3<f32>,
    // Material: kind, seed, board radius (tiles), unused. Constant per draw.
    @location(3) mat: vec4<f32>,
};

// Material kinds (`HuntLocal::mat.x`); see `gpu::board::Material`.
const MAT_MOSS: f32 = 1.0;
const MAT_MUD: f32 = 2.0;
const MAT_STONE: f32 = 3.0;
const MAT_WATER: f32 = 4.0;
const MAT_GROUND: f32 = 5.0;
const MAT_ROCK: f32 = 6.0;
const FOG: vec3<f32> = vec3<f32>(0.010, 0.011, 0.014);

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
    // Camera-relative linear depth, perpendicular to both isometric screen axes.
    // CPU bounds cover the whole scene; clamping would collapse its occlusion.
    let depth = 0.5 - dot(p - globals.depth_origin_scale.xyz, vec3<f32>(1.0))
        * globals.depth_origin_scale.w;
    let n = normalize(n0);
    let sun = max(dot(n, normalize(globals.light_dir.xyz)), 0.0);
    var lit = locals.color.xyz * (0.16 + 0.62 * sun);
    lit += lamp(p, n, globals.lamp0, globals.lamp0c);
    lit += lamp(p, n, globals.lamp1, globals.lamp1c);
    lit += locals.color.xyz * locals.pose.w;
    // Silhouette rim: the iso camera looks down -(1, 1, 1), so normals
    // perpendicular to that axis sit on the outline. Per-vertex like the rest
    // of the lighting (plain math, fine for GLSL ES 3.0 / WebGL2).
    let facing = abs(dot(n, vec3<f32>(0.57735027)));
    let edge = 1.0 - facing;
    lit += locals.rim.xyz * (locals.rim.w * edge * edge);
    // Cool sky fill from above so nothing sits in a dead-black studio void.
    lit += locals.color.xyz * vec3<f32>(0.07, 0.08, 0.10) * (0.6 + 0.4 * n.y);
    var o: VSOut;
    // The nest floor sits below every board cell and fighter: pin it to the
    // far plane so its huge extent never touches the board's depth range.
    var z = depth;
    if (abs(locals.mat.x - MAT_GROUND) < 0.5) {
        z = 0.9995;
    }
    o.clip = vec4<f32>(ndc_x, ndc_y, z, 1.0);
    o.color = vec4<f32>(lit, 1.0);
    let inv = 1.0 / max(locals.mat.w, 0.0001);
    o.rel = vec4<f32>((p - globals.depth_origin_scale.xyz) * inv, globals.screen.z);
    o.normal = n;
    o.mat = locals.mat;
    return o;
}

fn hash2(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

fn vnoise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2<f32>(1.0, 0.0));
    let c = hash2(i + vec2<f32>(0.0, 1.0));
    let d = hash2(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn fbm(p: vec2<f32>) -> f32 {
    return vnoise(p) * 0.55 + vnoise(p * 2.03 + 17.0) * 0.3 + vnoise(p * 4.11 + 41.0) * 0.15;
}

// Albedo multiplier for a terrain top face; the CPU tint (incl. occupancy)
// stays the base hue, so move / threat / hover read through the texture.
fn terrain_top(kind: f32, uv: vec2<f32>, t: f32) -> vec3<f32> {
    let n = fbm(uv * 2.6);
    let fine = vnoise(uv * 11.0);
    if (kind < MAT_MUD - 0.5) {
        // Moss: clumps of lighter growth over dark root mats, fine speckle.
        let clump = smoothstep(0.35, 0.75, n);
        return mix(vec3<f32>(0.55, 0.62, 0.50), vec3<f32>(1.25, 1.30, 1.05), clump) * (0.82 + 0.3 * fine);
    }
    if (kind < MAT_STONE - 0.5) {
        // Mud: churned earth, dark wet puddles with a faint sheen.
        let wet = smoothstep(0.58, 0.68, n);
        let base = vec3<f32>(0.75 + 0.45 * fine);
        return mix(base, vec3<f32>(0.42, 0.40, 0.44), wet) + vec3<f32>(0.10) * wet * fine;
    }
    if (kind < MAT_WATER - 0.5) {
        // Stone: worn flagstones split by dark cracks.
        let crack = 1.0 - smoothstep(0.0, 0.05, abs(vnoise(uv * 3.2 + 5.0) - 0.5));
        let slab = 0.78 + 0.35 * n + 0.12 * fine;
        return vec3<f32>(slab * (1.0 - 0.65 * crack));
    }
    // Water: still black pool with slow ripples catching the light.
    let ripple = sin(uv.x * 7.0 + uv.y * 3.0 + n * 6.0 + t * 1.3) * 0.5 + 0.5;
    return vec3<f32>(0.7 + 0.25 * n) + vec3<f32>(0.25, 0.30, 0.38) * pow(ripple, 8.0);
}

@fragment
fn fs_main(v: VSOut) -> @location(0) vec4<f32> {
    let kind = v.mat.x;
    if (kind < 0.5) {
        return v.color;
    }
    let uv = v.rel.xz + vec2<f32>(v.mat.y * 7.3, v.mat.y * 3.1);
    let dist = length(v.rel.xz);
    let radius = max(v.mat.z, 1.0);
    // Ground fog: the lit clearing fades into mist a few tiles past the dais.
    let mist = vnoise(v.rel.xz * 0.35 + vec2<f32>(v.rel.w * 0.05, v.rel.w * 0.02));
    let fog = clamp(smoothstep(radius + 1.0, radius + 9.0, dist) + (mist - 0.5) * 0.35, 0.0, 1.0);
    if (kind > MAT_GROUND - 0.5 && kind < MAT_ROCK - 0.5) {
        // Nest floor: dark loam with moss and pale bone grit, lit only near
        // the board (lamp pool) and swallowed by fog beyond.
        let n = fbm(v.rel.xz * 0.9);
        let moss = smoothstep(0.55, 0.8, fbm(v.rel.xz * 0.4 + 9.0));
        let grit = step(0.93, vnoise(v.rel.xz * 9.0));
        var alb = mix(vec3<f32>(0.026, 0.021, 0.017), vec3<f32>(0.016, 0.026, 0.012), moss) * (0.6 + 0.8 * n);
        alb += vec3<f32>(0.03, 0.028, 0.025) * grit;
        let pool = 1.0 - smoothstep(radius * 0.6, radius + 6.0, dist);
        let lit = alb * (0.35 + 2.0 * pool);
        return vec4<f32>(mix(lit, FOG, fog), 1.0);
    }
    var m: vec3<f32>;
    if (kind > MAT_ROCK - 0.5) {
        // Dais / boulders: rough dark stone, strata on the sides.
        let n = fbm(vec2<f32>(v.rel.x + v.rel.z, v.rel.y * 3.0) * 1.7 + uv);
        m = vec3<f32>(0.65 + 0.6 * n);
    } else if (v.normal.y > 0.5) {
        m = terrain_top(kind, uv, v.rel.w);
    } else {
        // Tile sides: packed earth strata under every terrain kind.
        let n = vnoise(vec2<f32>(v.rel.x + v.rel.z, v.rel.y * 9.0) * 2.0);
        m = vec3<f32>(0.62, 0.55, 0.50) * (0.7 + 0.5 * n);
    }
    var c = v.color.xyz * m;
    if (kind > MAT_ROCK - 0.5) {
        c = mix(c, FOG, fog);
    }
    return vec4<f32>(c, 1.0);
}
