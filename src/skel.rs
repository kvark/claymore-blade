//! Procedural combat skeleton. Blade's compute skinning is Vulkan/Metal-only;
//! WebGL2/GLES gets the same poses as linked hex-prisms.

use crate::combat::Side;

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
    let cloth = [
        (skin[0] * 0.72).min(1.0),
        (skin[1] * 0.72).min(1.0),
        (skin[2] * 0.72).min(1.0),
    ];
    let head_c = [
        (skin[0] * 1.08).min(1.0),
        (skin[1] * 1.04).min(1.0),
        (skin[2] * 0.96).min(1.0),
    ];

    [
        Bone {
            x: p.x,
            y: hip_y,
            z: p.z,
            radius: s * 0.16,
            height: hip_h,
            yaw: face,
            lean: 0.0,
            rgb: cloth,
            glow: glow * 0.3,
        },
        Bone {
            x: p.x + fwd_x * s * 0.02,
            y: torso_y,
            z: p.z + fwd_z * s * 0.02,
            radius: s * 0.14,
            height: torso_h,
            yaw: face,
            lean: sway * 0.15,
            rgb: skin,
            glow,
        },
        Bone {
            x: p.x + fwd_x * s * 0.03,
            y: head_y,
            z: p.z + fwd_z * s * 0.03,
            radius: s * 0.10,
            height: head_h,
            yaw: face,
            lean: sway * 0.08,
            rgb: head_c,
            glow: glow * 0.6 + act * 0.2,
        },
        Bone {
            x: p.x + right_x * arm_span + fwd_x * r_swing,
            y: torso_y + raise * 0.4,
            z: p.z + right_z * arm_span + fwd_z * r_swing,
            radius: s * 0.06,
            height: arm_h,
            yaw: face + 0.35 + act * 0.4,
            lean: -0.2 - act * 0.3,
            rgb: cloth,
            glow: act * 0.35,
        },
        Bone {
            x: p.x - right_x * arm_span + fwd_x * l_swing,
            y: torso_y,
            z: p.z - right_z * arm_span + fwd_z * l_swing,
            radius: s * 0.06,
            height: arm_h,
            yaw: face - 0.35,
            lean: 0.15,
            rgb: cloth,
            glow: 0.0,
        },
        Bone {
            x: p.x + right_x * leg_span + fwd_x * l_swing * 0.6,
            y: limb_h * 0.45 + bob * 0.3,
            z: p.z + right_z * leg_span + fwd_z * l_swing * 0.6,
            radius: s * 0.07,
            height: limb_h,
            yaw: face,
            lean: step * 0.2,
            rgb: cloth,
            glow: 0.0,
        },
        Bone {
            x: p.x - right_x * leg_span + fwd_x * r_swing * 0.6,
            y: limb_h * 0.45 + bob * 0.3,
            z: p.z - right_z * leg_span + fwd_z * r_swing * 0.6,
            radius: s * 0.07,
            height: limb_h,
            yaw: face,
            lean: -step * 0.2,
            rgb: cloth,
            glow: 0.0,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fighter_stays_on_feet() {
        let bones = pose_fighter(&PoseInput {
            x: 0.0,
            z: 0.0,
            size: 40.0,
            facing: 1,
            cam_yaw: 0,
            time: 0.7,
            color: [0.8, 0.7, 0.5],
            side: Side::Player,
            acting: true,
            hurt: false,
            trans: 20,
        });
        assert!(bones.iter().all(|b| b.height > 0.0 && b.radius > 0.0));
        assert!(bones[2].y > bones[0].y, "head sits above hip");
    }
}
