//! Procedural combat skeleton. GLES uses Blade's vertex-stage LBS path
//! (skin.inc-style joint palette + packed joints/weights).

use crate::combat::Side;
use crate::fx::FightClip;

#[derive(Clone, Copy)]
pub struct Bone {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub radius: f32,
    pub height: f32,
    pub yaw: f32,
    pub lean: f32,
    pub rgb: [f32; 3],
    pub glow: f32,
}

#[derive(Clone, Copy)]
pub struct PoseInput {
    pub x: f32,
    pub z: f32,
    pub size: f32,
    pub facing: i32,
    pub cam_yaw: u8,
    pub time: f32,
    pub color: [f32; 3],
    pub side: Side,
    pub acting: bool,
    pub hurt: bool,
    pub trans: i32,
    pub clip: FightClip,
    pub clip_u: f32,
}

/// 7-bone humanoid: hip, torso, head, two arms, two legs.
pub fn pose_fighter(p: &PoseInput) -> [Bone; 7] {
    let s = p.size;
    let face = (p.facing.rem_euclid(6) as f32) * std::f32::consts::FRAC_PI_3
        + (p.cam_yaw as f32) * std::f32::consts::FRAC_PI_2;
    let t = p.time;
    let act = if p.acting { 1.0 } else { 0.0 };
    let hurt = if p.hurt { 1.0 } else { 0.0 };
    let enemy = if p.side == Side::Enemy { 1.0 } else { 0.0 };
    // Silver-eyed stillness vs yoma twitch — not a shared drunken bob.
    let bob_amp = if enemy > 0.0 {
        s * (0.028 + act * 0.032)
    } else {
        s * (0.008 + act * 0.012)
    };
    let bob_freq = if enemy > 0.0 { 4.8 + act * 2.8 } else { 2.2 + act * 1.4 };
    let bob = (t * bob_freq).sin() * bob_amp;
    let sway_amp = if enemy > 0.0 {
        0.22 + act * 0.26
    } else {
        0.035 + act * 0.07
    };
    let sway_freq = if enemy > 0.0 { 3.5 } else { 1.55 };
    let sway = (t * sway_freq).sin() * sway_amp;
    let twitch = if enemy > 0.0 {
        (t * 11.0).sin() * 0.10 + (t * 7.1).cos() * 0.06
    } else {
        (t * 1.05).sin() * 0.012
    };
    let step = (t * (4.0 + act * 3.0 + enemy * 1.2)).sin();
    let recoil = hurt * (t * 18.0).sin().abs() * s * (0.04 + enemy * 0.02);
    let hip_h = s * (0.10 + enemy * 0.02);
    let torso_h = s * (0.16 + enemy * 0.04);
    let head_h = s * 0.10;
    let limb_h = s * (0.14 + enemy * 0.03);
    let arm_h = s * (0.12 + act * 0.02);
    let fwd_x = face.cos();
    let fwd_z = face.sin();
    let right_x = -fwd_z;
    let right_z = fwd_x;
    let hip_y = hip_h * 0.5 + bob - recoil;
    let torso_y = hip_y + hip_h * 0.45 + torso_h * 0.45;
    let head_y = torso_y + torso_h * 0.45 + head_h * 0.4;
    let arm_span = s * (0.16 + enemy * 0.04);
    let leg_span = s * 0.08;
    let l_swing = step * s * 0.06;
    let r_swing = -step * s * 0.06;
    let raise = act * s * 0.08;
    let glow = ((p.trans as f32) / 100.0).clamp(0.0, 1.0) * 0.55 + act * 0.25;
    let skin = p.color;
    let cloth = [(skin[0] * 0.72).min(1.0), (skin[1] * 0.72).min(1.0), (skin[2] * 0.72).min(1.0)];
    let head_c = [(skin[0] * 1.08).min(1.0), (skin[1] * 1.04).min(1.0), (skin[2] * 0.96).min(1.0)];
    let mut bones = [
        Bone { x: p.x, y: hip_y, z: p.z, radius: s * 0.16, height: hip_h, yaw: face, lean: 0.0, rgb: cloth, glow: glow * 0.3 },
        Bone { x: p.x + fwd_x * s * 0.02, y: torso_y, z: p.z + fwd_z * s * 0.02, radius: s * 0.14, height: torso_h, yaw: face, lean: sway * 0.15, rgb: skin, glow },
        Bone { x: p.x + fwd_x * s * 0.03, y: head_y, z: p.z + fwd_z * s * 0.03, radius: s * 0.10, height: head_h, yaw: face + twitch * 0.15, lean: sway * 0.08 + twitch * 0.05, rgb: head_c, glow: glow * 0.6 + act * 0.2 },
        Bone { x: p.x + right_x * arm_span + fwd_x * r_swing, y: torso_y + raise * 0.4, z: p.z + right_z * arm_span + fwd_z * r_swing, radius: s * 0.06, height: arm_h, yaw: face + 0.35 + act * 0.4, lean: -0.2 - act * 0.3, rgb: cloth, glow: act * 0.35 },
        Bone { x: p.x - right_x * arm_span + fwd_x * l_swing, y: torso_y, z: p.z - right_z * arm_span + fwd_z * l_swing, radius: s * 0.06, height: arm_h, yaw: face - 0.35, lean: 0.15, rgb: cloth, glow: 0.0 },
        Bone { x: p.x + right_x * leg_span + fwd_x * l_swing * 0.6, y: limb_h * 0.45 + bob * 0.3, z: p.z + right_z * leg_span + fwd_z * l_swing * 0.6, radius: s * 0.07, height: limb_h, yaw: face, lean: step * 0.2, rgb: cloth, glow: 0.0 },
        Bone { x: p.x - right_x * leg_span + fwd_x * r_swing * 0.6, y: limb_h * 0.45 + bob * 0.3, z: p.z - right_z * leg_span + fwd_z * r_swing * 0.6, radius: s * 0.07, height: limb_h, yaw: face, lean: -step * 0.2, rgb: cloth, glow: 0.0 },
    ];
    apply_clip(&mut bones, p.clip, p.clip_u, s, fwd_x, fwd_z, p.side);
    bones
}

fn pulse(u: f32, peak: f32) -> f32 {
    if u <= peak {
        (u / peak.max(0.001)).clamp(0.0, 1.0)
    } else {
        (1.0 - (u - peak) / (1.0 - peak).max(0.001)).clamp(0.0, 1.0)
    }
}

fn apply_clip(
    bones: &mut [Bone; 7],
    clip: FightClip,
    u: f32,
    s: f32,
    fwd_x: f32,
    fwd_z: f32,
    side: Side,
) {
    let u = u.clamp(0.0, 1.0);
    let enemy = side == Side::Enemy;
    match clip {
        FightClip::Idle | FightClip::Ready => {}
        FightClip::Slash => {
            if enemy {
                // Yoma anatomy: stretchier limbs, heavier plant/recoil.
                let k = pulse(u, 0.42);
                let stretch = k * s * 0.40;
                bones[1].yaw += k * 0.48;
                bones[1].lean += k * 0.24;
                bones[3].x += fwd_x * stretch;
                bones[3].z += fwd_z * stretch;
                bones[3].y += k * s * 0.11;
                bones[3].height *= 1.0 + k * 0.55;
                bones[3].radius *= 1.0 + k * 0.18;
                bones[4].height *= 1.0 + k * 0.30;
                bones[3].yaw += k * 0.65;
                bones[3].lean -= k * 0.38;
                bones[3].glow = (bones[3].glow + k * 0.55).min(1.2);
                bones[0].y -= k * s * 0.05;
                bones[0].x -= fwd_x * k * s * 0.07;
                bones[0].z -= fwd_z * k * s * 0.07;
            } else {
                // Quicksword snap: early peak, short reach, sharp arm/torso whip.
                let k = pulse(u, 0.20);
                let reach = k * s * 0.15;
                bones[1].yaw += k * 0.62;
                bones[1].lean += k * 0.05;
                bones[1].x += fwd_x * reach * 0.35;
                bones[1].z += fwd_z * reach * 0.35;
                bones[3].x += fwd_x * reach;
                bones[3].z += fwd_z * reach;
                bones[3].y += k * s * 0.025;
                bones[3].yaw += k * 1.25;
                bones[3].lean -= k * 0.60;
                bones[3].glow = (bones[3].glow + k * 0.85).min(1.2);
                bones[4].x -= fwd_x * reach * 0.40;
                bones[4].z -= fwd_z * reach * 0.40;
                bones[2].yaw += k * 0.12;
            }
        }

        FightClip::Flash => {
            // Quicksword: blink step-in + silver blade burst — not a Cut arm-whip.
            let k = pulse(u, 0.12);
            let step = k * s * (0.30 + if enemy { 0.06 } else { 0.0 });
            for b in bones.iter_mut() {
                b.x += fwd_x * step;
                b.z += fwd_z * step;
            }
            bones[0].y -= k * s * 0.018;
            // Thrust the blade forward with a hard silver flare (glow), little yaw whip.
            bones[3].x += fwd_x * k * s * 0.10;
            bones[3].z += fwd_z * k * s * 0.10;
            bones[3].y += k * s * 0.05;
            bones[3].lean -= k * 0.28;
            bones[3].glow = (bones[3].glow + k * 1.15).min(1.4);
            bones[1].glow = (bones[1].glow + k * 0.55).min(1.25);
            bones[1].yaw += k * 0.18;
            bones[4].x -= fwd_x * k * s * 0.04;
            bones[4].z -= fwd_z * k * s * 0.04;
        }

        FightClip::Lunge => {
            let k = pulse(u, 0.45);
            let reach = k * s * (0.20 + if enemy { 0.08 } else { 0.0 });
            for b in bones.iter_mut() {
                b.x += fwd_x * reach;
                b.z += fwd_z * reach;
            }
            bones[0].y -= k * s * 0.03;
            bones[5].y -= k * s * 0.02;
            if enemy {
                bones[3].height *= 1.0 + k * 0.25;
            }
        }
        FightClip::Guard => {
            // Claymore "blade up": braced plant, sword high and upright — not a crouch.
            let k = pulse(u, 0.18).max(0.72);
            bones[0].y -= k * s * 0.012;
            bones[0].x += fwd_x * k * s * 0.02;
            bones[0].z += fwd_z * k * s * 0.02;
            bones[1].lean -= k * 0.06;
            bones[1].yaw += k * 0.08;
            bones[2].lean -= k * 0.04;
            // Weapon arm: raise and tip the blade vertical in front of the body.
            bones[3].y += k * s * 0.14;
            bones[3].x += fwd_x * s * 0.06 * k;
            bones[3].z += fwd_z * s * 0.06 * k;
            bones[3].yaw -= k * 0.55;
            bones[3].lean -= k * 0.85;
            bones[3].glow = (bones[3].glow + k * 0.35).min(1.2);
            // Off-hand braces near the hilt / chest.
            bones[4].y += k * s * 0.06;
            bones[4].x += fwd_x * s * 0.05 * k;
            bones[4].z += fwd_z * s * 0.05 * k;
            bones[4].lean -= k * 0.25;
            bones[5].lean += k * 0.08;
            bones[6].lean -= k * 0.06;
        }
        FightClip::Wait => {
            // Still breath, sword lowered — not a Ready dance or idle bob amp.
            let k = pulse(u, 0.25).max(0.65);
            let breath = (u * std::f32::consts::PI).sin().abs() * 0.35 + 0.65;
            let hold = k * breath;
            bones[1].y += hold * s * 0.012;
            bones[2].y += hold * s * 0.008;
            // Lower the claymore; soften the ready arm angle.
            bones[3].y -= k * s * 0.07;
            bones[3].x -= fwd_x * s * 0.03 * k;
            bones[3].z -= fwd_z * s * 0.03 * k;
            bones[3].yaw -= k * 0.28;
            bones[3].lean += k * 0.22;
            bones[4].y -= k * s * 0.02;
            bones[4].lean += k * 0.08;
            // Plant: kill the walk swing read without squatting.
            bones[5].lean *= 1.0 - k * 0.55;
            bones[6].lean *= 1.0 - k * 0.55;
        }
        FightClip::Raise => {
            let k = pulse(u, 0.4).max(0.4);
            bones[3].y += k * s * 0.16;
            bones[4].y += k * s * 0.14;
            bones[1].glow = (bones[1].glow + k * 0.5).min(1.2);
            bones[2].glow = (bones[2].glow + k * 0.4).min(1.2);
        }
        FightClip::Hurt => {
            // Real flinch: crumple, twist, head tuck, arm flail — not a tint.
            let k = pulse(u, 0.20);
            let impact = if enemy { 1.45 } else { 1.0 };
            let back = k * s * 0.24 * impact;
            for b in bones.iter_mut() {
                b.x -= fwd_x * back;
                b.z -= fwd_z * back;
            }
            bones[0].y -= k * s * 0.055 * impact;
            bones[1].lean -= k * 0.48 * impact;
            bones[1].yaw += k * 0.40 * impact;
            bones[2].y -= k * s * 0.065 * impact;
            bones[2].lean -= k * 0.35 * impact;
            bones[2].yaw -= k * 0.20;
            bones[3].y -= k * s * 0.05;
            bones[3].lean += k * 0.55 * impact;
            bones[3].x -= fwd_x * k * s * 0.04;
            bones[3].z -= fwd_z * k * s * 0.04;
            bones[4].y += k * s * 0.035;
            bones[4].lean -= k * 0.45;
            bones[5].lean += k * 0.40 * impact;
            bones[6].lean -= k * 0.30 * impact;
            if enemy {
                bones[3].height *= 1.0 + k * 0.20;
                bones[4].height *= 1.0 + k * 0.15;
            }
        }
    }
}

pub const JOINTS: usize = 8;

pub fn bind_centers() -> [[f32; 3]; 7] {
    [
        [0.0, 0.10, 0.0],
        [0.0, 0.28, 0.0],
        [0.0, 0.48, 0.0],
        [0.16, 0.30, 0.0],
        [-0.16, 0.30, 0.0],
        [0.08, 0.08, 0.0],
        [-0.08, 0.08, 0.0],
    ]
}

fn affine(tx: f32, ty: f32, tz: f32, yaw: f32, sx: f32, sy: f32, sz: f32) -> [f32; 12] {
    let c = yaw.cos();
    let s = yaw.sin();
    [c * sx, 0.0, -s * sz, tx, 0.0, sy, 0.0, ty, s * sx, 0.0, c * sz, tz]
}

pub fn joint_palette(bones: &[Bone; 7]) -> [[f32; 12]; JOINTS] {
    let bind = bind_centers();
    let mut out = [[0.0; 12]; JOINTS];
    out[7] = affine(0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    for i in 0..7 {
        let b = bones[i];
        let p = bind[i];
        let c = b.yaw.cos();
        let s = b.yaw.sin();
        let sx = b.radius.max(0.001);
        let sy = b.height.max(0.001);
        let sz = sx;
        let r00 = c * sx;
        let r02 = -s * sz;
        let r20 = s * sx;
        let r22 = c * sz;
        let r11 = sy;
        let tx = b.x - (r00 * p[0] + r02 * p[2]);
        let ty = b.y - r11 * p[1];
        let tz = b.z - (r20 * p[0] + r22 * p[2]);
        out[i] = [r00, 0.0, r02, tx, 0.0, r11, 0.0, ty, r20, 0.0, r22, tz];
    }
    out
}

pub fn identity_palette() -> [[f32; 12]; JOINTS] {
    let mut out = [[0.0; 12]; JOINTS];
    let id = affine(0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0);
    for slot in &mut out {
        *slot = id;
    }
    out
}

/// Bind centers for height-normalized archive GLBs (Y in `[0, 1]`, feet on ground).
pub fn archive_bind_centers() -> [[f32; 3]; 7] {
    [
        [0.0, 0.18, 0.0],
        [0.0, 0.48, 0.0],
        [0.0, 0.88, 0.0],
        [0.22, 0.55, 0.0],
        [-0.22, 0.55, 0.0],
        [0.09, 0.12, 0.0],
        [-0.09, 0.12, 0.0],
    ]
}

/// Map a height-normalized archive vertex to the 7-bone fight palette.
/// Height bands first (works for asymmetric meshes like vika); extreme
/// lateral verts become arms so wide monsters (ifrit/valefor) can swing.
pub fn assign_archive_joint(pos: [f32; 3]) -> u32 {
    let x = pos[0];
    let y = pos[1];
    if y < 0.26 {
        if x.abs() > 0.05 {
            return if x >= 0.0 { 5 } else { 6 };
        }
        return 0;
    }
    // Arms/wings before head. Lower+widen vs old (0.42..0.70, |x|>0.40):
    // vika max |x|≈0.36 so sleeves/blade bind; valefor wings leave the crown.
    if (0.36..=0.90).contains(&y) && x.abs() > 0.30 {
        return if x >= 0.0 { 3 } else { 4 };
    }
    if y > 0.82 {
        return 2;
    }
    if y < 0.42 {
        return 0;
    }
    1
}

fn rel_yaw_affine(bind: [f32; 3], dx: f32, dy: f32, dz: f32, yaw: f32) -> [f32; 12] {
    let c = yaw.cos();
    let s = yaw.sin();
    // p' = R_y(yaw) * (p - bind) + bind + delta
    let tx = bind[0] - (c * bind[0] - s * bind[2]) + dx;
    let ty = dy;
    let tz = bind[2] - (s * bind[0] + c * bind[2]) + dz;
    [c, 0.0, -s, tx, 0.0, 1.0, 0.0, ty, s, 0.0, c, tz]
}

/// Model-space relative skin from `pose_fighter` deltas (idle bob, Cut swing, hurt flinch).
/// Archive meshes stay in unit height; world/pose still place and face the draw.
pub fn archive_joint_palette(p: &PoseInput) -> [[f32; 12]; JOINTS] {
    let posed = pose_fighter(p);
    let rest = pose_fighter(&PoseInput {
        x: p.x,
        z: p.z,
        size: p.size,
        facing: p.facing,
        cam_yaw: p.cam_yaw,
        time: 0.0,
        color: p.color,
        side: p.side,
        acting: false,
        hurt: false,
        trans: p.trans,
        clip: FightClip::Idle,
        clip_u: 0.0,
    });
    let s = p.size.max(1.0);
    let bind = archive_bind_centers();
    let mut out = identity_palette();
    let enemy = p.side == Side::Enemy;
    // Slim archive heroes still fold some weapon-arm reach into torso; real arm
    // verts (after widened band) carry the Quicksword snap themselves.
    let arm_blend = if enemy { 0.35 } else { 0.40 };
    let weapon_dx = (posed[3].x - rest[3].x) / s;
    let weapon_dy = (posed[3].y - rest[3].y) / s;
    let weapon_dz = (posed[3].z - rest[3].z) / s;
    let weapon_dyaw = posed[3].yaw - rest[3].yaw;
    let (mx, my, mz, myaw) = match p.clip {
        FightClip::Slash if !enemy => (1.15, 1.08, 1.15, 1.45), // punch yaw for snap
        FightClip::Slash if enemy => (1.55, 1.40, 1.55, 1.20),  // stretch read
        FightClip::Flash => (1.55, 1.20, 1.55, 0.85), // step-in read, soft yaw
        FightClip::Hurt => (1.50, 1.40, 1.50, 1.30),
        FightClip::Guard => (1.20, 1.45, 1.20, 1.25), // blade-up vertical read
        FightClip::Wait => (1.15, 1.35, 1.15, 1.15),  // lowered sword
        _ => (1.25, 1.15, 1.25, 1.10),
    };
    for i in 0..7 {
        let mut dx = (posed[i].x - rest[i].x) / s;
        let mut dy = (posed[i].y - rest[i].y) / s;
        let mut dz = (posed[i].z - rest[i].z) / s;
        let mut dyaw = posed[i].yaw - rest[i].yaw;
        if i == 1 {
            dx += weapon_dx * arm_blend;
            dy += weapon_dy * arm_blend * 0.5;
            dz += weapon_dz * arm_blend;
            dyaw += weapon_dyaw * 0.35;
        }
        // vika mass sits on −x (joint 4); drive both arm slots from weapon arm
        // so Quicksword snap reads on whichever sleeve the mesh put verts on.
        if i == 3 || i == 4 {
            dx = weapon_dx;
            dy = weapon_dy;
            dz = weapon_dz;
            dyaw = weapon_dyaw;
        }
        dx *= mx;
        dy *= my;
        dz *= mz;
        dyaw *= myaw;
        out[i] = rel_yaw_affine(bind[i], dx, dy, dz, dyaw);
    }
    out
}

pub fn fighter_vertices() -> Vec<([f32; 3], [f32; 3], u32, u32)> {
    let bind = bind_centers();
    let half = [
        [0.16, 0.10, 0.12],
        [0.14, 0.16, 0.11],
        [0.10, 0.10, 0.09],
        [0.06, 0.12, 0.05],
        [0.06, 0.12, 0.05],
        [0.07, 0.14, 0.06],
        [0.07, 0.14, 0.06],
    ];
    let mut out = Vec::new();
    for i in 0..7 {
        push_box(&mut out, bind[i], half[i], i as u32);
    }
    out
}

fn push_box(out: &mut Vec<([f32; 3], [f32; 3], u32, u32)>, c: [f32; 3], h: [f32; 3], joint: u32) {
    let corners = [
        [c[0] - h[0], c[1] - h[1], c[2] - h[2]],
        [c[0] + h[0], c[1] - h[1], c[2] - h[2]],
        [c[0] + h[0], c[1] + h[1], c[2] - h[2]],
        [c[0] - h[0], c[1] + h[1], c[2] - h[2]],
        [c[0] - h[0], c[1] - h[1], c[2] + h[2]],
        [c[0] + h[0], c[1] - h[1], c[2] + h[2]],
        [c[0] + h[0], c[1] + h[1], c[2] + h[2]],
        [c[0] - h[0], c[1] + h[1], c[2] + h[2]],
    ];
    let faces: [([usize; 4], [f32; 3]); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0]),
        ([5, 4, 7, 6], [0.0, 0.0, 1.0]),
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0]),
        ([1, 5, 6, 2], [1.0, 0.0, 0.0]),
        ([3, 2, 6, 7], [0.0, 1.0, 0.0]),
        ([4, 5, 1, 0], [0.0, -1.0, 0.0]),
    ];
    let w = 0x0000_00FFu32;
    for (idx, n) in faces {
        let tri = [idx[0], idx[1], idx[2], idx[0], idx[2], idx[3]];
        for i in tri {
            out.push((corners[i], n, joint, w));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fighter_stays_on_feet() {
        let bones = pose_fighter(&PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 1, cam_yaw: 0, time: 0.7,
            color: [0.8, 0.7, 0.5], side: Side::Player, acting: true, hurt: false, trans: 20,
            clip: FightClip::Idle, clip_u: 0.0,
        });
        assert!(bones.iter().all(|b| b.height > 0.0 && b.radius > 0.0));
        assert!(bones[2].y > bones[0].y, "head sits above hip");
        assert_eq!(fighter_vertices().len() % 3, 0);
    }

    #[test]
    fn slash_reaches_forward() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: false, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = pose_fighter(&base);
        // Quicksword peaks early (~0.20); still a forward snap, not a wide slow arc.
        let slash = pose_fighter(&PoseInput { clip: FightClip::Slash, clip_u: 0.20, ..base });
        let dx = slash[3].x - idle[3].x;
        let dz = slash[3].z - idle[3].z;
        assert!(dx * dx + dz * dz > 12.0, "weapon arm should snap forward on slash");
        assert!(
            (slash[3].yaw - idle[3].yaw).abs() > 0.8,
            "Quicksword should whip arm yaw hard"
        );
        let yoma = pose_fighter(&PoseInput {
            side: Side::Enemy,
            clip: FightClip::Slash,
            clip_u: 0.42,
            ..base
        });
        let ydx = yoma[3].x - idle[3].x;
        let ydz = yoma[3].z - idle[3].z;
        assert!(
            ydx * ydx + ydz * ydz > dx * dx + dz * dz,
            "yoma slash should stretch farther than Quicksword snap"
        );
        assert!(yoma[3].height > slash[3].height, "yoma arm should elongate on slash");
    }

    #[test]
    fn archive_joint_by_height() {
        assert_eq!(assign_archive_joint([0.0, 0.05, 0.0]), 0, "feet-center → hip");
        assert_eq!(assign_archive_joint([0.12, 0.10, 0.0]), 5, "right foot → right leg");
        assert_eq!(assign_archive_joint([-0.12, 0.10, 0.0]), 6, "left foot → left leg");
        assert_eq!(assign_archive_joint([0.0, 0.50, 0.0]), 1, "chest → torso");
        assert_eq!(assign_archive_joint([0.0, 0.92, 0.0]), 2, "crown → head");
        assert_eq!(assign_archive_joint([0.55, 0.55, 0.0]), 3, "wide right → arm");
        assert_eq!(assign_archive_joint([-0.55, 0.55, 0.0]), 4, "wide left → arm");
        // Widened band: vika-scale lateral mid verts bind to arms (|x|>0.30).
        assert_eq!(assign_archive_joint([-0.32, 0.60, 0.0]), 4, "vika sleeve → arm");
        assert_eq!(assign_archive_joint([0.35, 0.85, 0.0]), 3, "high wing/arm before head");
    }

    #[test]
    fn archive_slash_palette_moves_torso() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: true, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = archive_joint_palette(&base);
        let slash = archive_joint_palette(&PoseInput { clip: FightClip::Slash, clip_u: 0.20, ..base });
        // Torso translation tx/tz (indices 3 and 11) should shift on slash.
        let dtx = slash[1][3] - idle[1][3];
        let dtz = slash[1][11] - idle[1][11];
        assert!(dtx * dtx + dtz * dtz > 0.004, "archive torso should whip on slash");
        // Arm slots (both) should move — vika mass may sit on joint 4.
        let adx = slash[3][3] - idle[3][3];
        let adz = slash[3][11] - idle[3][11];
        let bdx = slash[4][3] - idle[4][3];
        let bdz = slash[4][11] - idle[4][11];
        assert!(adx * adx + adz * adz > 0.01, "weapon arm joint should snap");
        assert!(bdx * bdx + bdz * bdz > 0.01, "opposite arm slot should also snap");
        let hurt = archive_joint_palette(&PoseInput { clip: FightClip::Hurt, clip_u: 0.20, ..base });
        let htx = hurt[1][3] - idle[1][3];
        let htz = hurt[1][11] - idle[1][11];
        assert!(htx * htx + htz * htz > 0.01, "archive torso should flinch hard on hurt");
        let hlean = (hurt[1][0] - idle[1][0]).abs() + (hurt[2][7] - idle[2][7]).abs();
        let _ = hlean; // head/torso also shift in ty
        assert!((hurt[2][7] - idle[2][7]).abs() > 0.01, "head should tuck on hurt");
    }

    #[test]
    fn guard_raises_blade_wait_lowers() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: false, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = pose_fighter(&base);
        let guard = pose_fighter(&PoseInput { clip: FightClip::Guard, clip_u: 0.35, ..base });
        let wait = pose_fighter(&PoseInput { clip: FightClip::Wait, clip_u: 0.40, ..base });
        assert!(
            guard[3].y > idle[3].y + 2.0,
            "Guard should raise the weapon arm (blade up), got {} vs {}",
            guard[3].y,
            idle[3].y
        );
        assert!(
            (guard[3].lean - idle[3].lean).abs() > 0.4,
            "Guard blade should tip upright vs ready lean"
        );
        assert!(
            wait[3].y < idle[3].y - 1.0,
            "Wait should lower the sword, got {} vs {}",
            wait[3].y,
            idle[3].y
        );
        assert!(
            guard[3].y > wait[3].y + 4.0,
            "Guard and Wait must read as distinct arm heights"
        );
    }

    #[test]
    fn archive_guard_wait_palette() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: true, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = archive_joint_palette(&base);
        let guard = archive_joint_palette(&PoseInput { clip: FightClip::Guard, clip_u: 0.35, ..base });
        let wait = archive_joint_palette(&PoseInput { clip: FightClip::Wait, clip_u: 0.40, ..base });
        let gdy = guard[3][7] - idle[3][7];
        let wdy = wait[3][7] - idle[3][7];
        assert!(gdy > 0.02, "archive Guard should lift arm joint ty, got {gdy}");
        assert!(wdy < -0.01, "archive Wait should drop arm joint ty, got {wdy}");
        assert!(gdy > wdy + 0.04, "archive Guard/Wait arm heights must differ");
    }

    #[test]
    fn flash_steps_in_not_slash_whip() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: false, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = pose_fighter(&base);
        let slash = pose_fighter(&PoseInput { clip: FightClip::Slash, clip_u: 0.20, ..base });
        let flash = pose_fighter(&PoseInput { clip: FightClip::Flash, clip_u: 0.15, ..base });
        let flash_dx = flash[0].x - idle[0].x;
        let slash_dx = slash[0].x - idle[0].x;
        assert!(
            flash_dx.abs() > slash_dx.abs() + 1.0,
            "Flash should step the body in farther than Cut whip, flash={flash_dx} slash={slash_dx}"
        );
        assert!(
            flash[3].glow > slash[3].glow + 0.05,
            "Flash blade should silver-flare harder than Cut, flash={} slash={}",
            flash[3].glow,
            slash[3].glow
        );
        let slash_yaw = (slash[3].yaw - idle[3].yaw).abs();
        let flash_yaw = (flash[3].yaw - idle[3].yaw).abs();
        assert!(
            slash_yaw > flash_yaw + 0.2,
            "Cut should yaw-whip more than Flash thrust, slash={slash_yaw} flash={flash_yaw}"
        );
    }

    #[test]
    fn archive_flash_palette_steps() {
        let base = PoseInput {
            x: 0.0, z: 0.0, size: 40.0, facing: 0, cam_yaw: 0, time: 0.0,
            color: [1.0, 1.0, 1.0], side: Side::Player, acting: true, hurt: false, trans: 0,
            clip: FightClip::Idle, clip_u: 0.0,
        };
        let idle = archive_joint_palette(&base);
        let flash = archive_joint_palette(&PoseInput { clip: FightClip::Flash, clip_u: 0.15, ..base });
        let slash = archive_joint_palette(&PoseInput { clip: FightClip::Slash, clip_u: 0.20, ..base });
        // tx is index 3 in the 12-float affine
        let fdx = (flash[0][3] - idle[0][3]).abs();
        let sdx = (slash[0][3] - idle[0][3]).abs();
        assert!(fdx > sdx + 0.01, "archive Flash root should translate more than Slash, f={fdx} s={sdx}");
    }
}
