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
    let bob = (t * (3.1 + act * 2.4)).sin() * s * (0.018 + act * 0.02);
    let sway = (t * 2.2).sin() * (0.12 + act * 0.18);
    let step = (t * (4.0 + act * 3.0)).sin();
    let recoil = hurt * (t * 18.0).sin().abs() * s * 0.04;
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
        Bone { x: p.x + fwd_x * s * 0.03, y: head_y, z: p.z + fwd_z * s * 0.03, radius: s * 0.10, height: head_h, yaw: face, lean: sway * 0.08, rgb: head_c, glow: glow * 0.6 + act * 0.2 },
        Bone { x: p.x + right_x * arm_span + fwd_x * r_swing, y: torso_y + raise * 0.4, z: p.z + right_z * arm_span + fwd_z * r_swing, radius: s * 0.06, height: arm_h, yaw: face + 0.35 + act * 0.4, lean: -0.2 - act * 0.3, rgb: cloth, glow: act * 0.35 },
        Bone { x: p.x - right_x * arm_span + fwd_x * l_swing, y: torso_y, z: p.z - right_z * arm_span + fwd_z * l_swing, radius: s * 0.06, height: arm_h, yaw: face - 0.35, lean: 0.15, rgb: cloth, glow: 0.0 },
        Bone { x: p.x + right_x * leg_span + fwd_x * l_swing * 0.6, y: limb_h * 0.45 + bob * 0.3, z: p.z + right_z * leg_span + fwd_z * l_swing * 0.6, radius: s * 0.07, height: limb_h, yaw: face, lean: step * 0.2, rgb: cloth, glow: 0.0 },
        Bone { x: p.x - right_x * leg_span + fwd_x * r_swing * 0.6, y: limb_h * 0.45 + bob * 0.3, z: p.z - right_z * leg_span + fwd_z * r_swing * 0.6, radius: s * 0.07, height: limb_h, yaw: face, lean: -step * 0.2, rgb: cloth, glow: 0.0 },
    ];
    apply_clip(&mut bones, p.clip, p.clip_u, s, fwd_x, fwd_z);
    bones
}

fn pulse(u: f32, peak: f32) -> f32 {
    if u <= peak {
        (u / peak.max(0.001)).clamp(0.0, 1.0)
    } else {
        (1.0 - (u - peak) / (1.0 - peak).max(0.001)).clamp(0.0, 1.0)
    }
}

fn apply_clip(bones: &mut [Bone; 7], clip: FightClip, u: f32, s: f32, fwd_x: f32, fwd_z: f32) {
    let u = u.clamp(0.0, 1.0);
    match clip {
        FightClip::Idle | FightClip::Ready => {}
        FightClip::Slash => {
            let k = pulse(u, 0.38);
            let reach = k * s * 0.28;
            bones[1].yaw += k * 0.35;
            bones[1].lean += k * 0.12;
            bones[3].x += fwd_x * reach;
            bones[3].z += fwd_z * reach;
            bones[3].y += k * s * 0.06;
            bones[3].yaw += k * 0.8;
            bones[3].lean -= k * 0.45;
            bones[3].glow = (bones[3].glow + k * 0.7).min(1.2);
            bones[4].x -= fwd_x * reach * 0.25;
            bones[4].z -= fwd_z * reach * 0.25;
        }
        FightClip::Lunge => {
            let k = pulse(u, 0.45);
            let reach = k * s * 0.20;
            for b in bones.iter_mut() {
                b.x += fwd_x * reach;
                b.z += fwd_z * reach;
            }
            bones[0].y -= k * s * 0.03;
            bones[5].y -= k * s * 0.02;
        }
        FightClip::Guard => {
            let k = pulse(u, 0.2).max(0.55);
            bones[0].y -= k * s * 0.03;
            bones[3].x += fwd_x * s * 0.10 * k;
            bones[3].z += fwd_z * s * 0.10 * k;
            bones[4].x += fwd_x * s * 0.10 * k;
            bones[4].z += fwd_z * s * 0.10 * k;
            bones[3].y += k * s * 0.04;
            bones[4].y += k * s * 0.04;
        }
        FightClip::Raise => {
            let k = pulse(u, 0.4).max(0.4);
            bones[3].y += k * s * 0.16;
            bones[4].y += k * s * 0.14;
            bones[1].glow = (bones[1].glow + k * 0.5).min(1.2);
            bones[2].glow = (bones[2].glow + k * 0.4).min(1.2);
        }
        FightClip::Hurt => {
            let k = pulse(u, 0.25);
            let back = k * s * 0.14;
            for b in bones.iter_mut() {
                b.x -= fwd_x * back;
                b.z -= fwd_z * back;
            }
            bones[1].lean -= k * 0.25;
            bones[2].y -= k * s * 0.03;
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
        let slash = pose_fighter(&PoseInput { clip: FightClip::Slash, clip_u: 0.38, ..base });
        let dx = slash[3].x - idle[3].x;
        let dz = slash[3].z - idle[3].z;
        assert!(dx * dx + dz * dz > 20.0, "weapon arm should lunge on slash");
    }
}
