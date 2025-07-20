// src/gastly/constants.rs - ENHANCED AURA CONFIGURATION FOR GAPLESS SYSTEM

use smash::phx::Hash40;
use smash::hash40;
use once_cell::sync::Lazy;
use smash::lib::lua_const::*; // For FIGHTER_STATUS_KIND_ constants

pub const GASTLY_AURA_DEBUG_LOGGING: bool = false;
pub const GASTLY_AUDIO_DEBUG: bool = true; // Set to false to disable debug logs

pub const GASTLY_AURA_SCALE: f32 = 2.5;

// UI Cutin
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY: i32 = 0x200000F0;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY: i32 = 0x200000F1;

// CUTIN SYSTEM TOGGLE
pub const ENABLE_EVOLUTION_CUTINS: bool = false;  // Set to true to enable cutins

// Debug flag for evolving mesh system
pub const EVOLVING_MESH_DEBUG_LOGGING: bool = false;

// Gastly evolving meshes
pub static GASTLY_EVOLVING: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_evolving") });
pub static GASTLY_EVOLVING_FLOORSHADOW: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_evolvingfloorshadow") });
pub static GASTLY_EVOLVING_SHADOWBALL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_evolvingshadowball") });
pub static GASTLY_EVOLVING_TONGUE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_evolvingtongue") });

// Haunter evolving meshes  
pub static HAUNTER_EVOLVING: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_evolving") });
pub static HAUNTER_EVOLVING_FLOORSHADOW: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_evolvingfloorshadow") });
pub static HAUNTER_EVOLVING_SHADOWBALL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_evolvingshadowball") });

// Looping sound flags + shiny effect
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE: i32 = 0x200000f8;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE: i32 = 0x200000f9;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE: i32 = 0x200000fa;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE: i32 = 0x200000fb;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE: i32 = 0x200000fc;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE: i32 = 0x200000fd;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE: i32 = 0x200000fe;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE: i32 = 0x200000ff;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE: i32 = 0x20000102;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE: i32 = 0x20000100;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_FURAFURA_ACTIVE: i32 = 0x20000101;

// Handle constants for one-shot sounds (used by persist_sfx.rs)
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SE_HANDLE: i32 = 0x50000025;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SS_HANDLE: i32 = 0x50000026;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_HANDLE: i32 = 0x50000027;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_X_HANDLE: i32 = 0x50000028;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_LINKING_CORD_HANDLE: i32 = 0x50000029;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_DYNAMAX_HANDLE: i32 = 0x5000002A;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_GENGARITE_HANDLE: i32 = 0x5000002B;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_CANCEL_EVOLVE_HANDLE: i32 = 0x5000002C;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_GRAB_BURN_HANDLE: i32 = 0x5000002D;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_MEGASYMBOL_HANDLE: i32 = 0x5000002E;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_POTION_HANDLE: i32 = 0x5000002F;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_RESTORE_HANDLE: i32 = 0x50000030;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_SHADOWBALL_HANDLE: i32 = 0x50000031;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_SHINY_SPARKLE_HANDLE: i32 = 0x50000032;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_INT_SHINY_EFFECT_HANDLE: i32 = 0x50000033;

// Looping sound timers
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER: i32 = 0x72;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER: i32 = 0x73;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER: i32 = 0x74;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER: i32 = 0x75;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_GRAB_BURN_TIMER: i32 = 0x76;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER: i32 = 0x77;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER: i32 = 0x78;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_FURAFURA_TIMER: i32 = 0x79;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER: i32 = 0x7A;

// --- Icon Meshes ---
pub static LINKING_CORD_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("linking_cord") });
pub static EVERSTONE_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("everstone") });
pub static EVERSTONE_X_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("everstone_x") });
pub static GENGARITE_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengarite") });
pub static DYNAMAX_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("dynamax") });
pub static BILLBOARD_BONE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("billboard") }); 

// --- Evolution Readiness Indicator Icons ---
pub static STG1_DMG_T_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("stg1_dmg_t") });
pub static STG1_DMG_D_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("stg1_dmg_d") });
pub static STG2_DMG_SS_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("stg2_dmg_ss") });
pub static STG2_DMG_SE_ICON: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("stg2_dmg_se") });

pub const READINESS_ICON_DURATION: i32 = 70; // Frames

// --- Character Mesh Hashes ---
pub static GASTLY_BODY: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_body") });
pub static GASTLY_TONGUE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_tongue") });
pub static GASTLY_EYE_N: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eyen") });
pub static GASTLY_EYE_BLINK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_blink") });
pub static GASTLY_EYE_HALFBLINK1: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_halfblink1") });
pub static GASTLY_EYE_ATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_attack") });
pub static GASTLY_EYE_CAPTURE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_capture") });
pub static GASTLY_EYE_OUCH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_ouch") });
pub static GASTLY_EYE_DOWN: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_down") });
pub static GASTLY_EYE_HEAVYATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_eye_heavyattack") });

pub static PURIN_VANILLA_EYE_N: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eyen") });
pub static PURIN_VANILLA_EYE_BLINK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_blink") });
pub static PURIN_VANILLA_EYE_HALFBLINK1: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_halfblink1") });
pub static PURIN_VANILLA_EYE_ATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_attack") });
pub static PURIN_VANILLA_EYE_CAPTURE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_capture") });
pub static PURIN_VANILLA_EYE_OUCH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_ouch") });
pub static PURIN_VANILLA_EYE_DOWN: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_down") });
pub static PURIN_VANILLA_EYE_HEAVYATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("purin_eye_heavyattack") });

pub static HAUNTER_BODY: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_body") });
pub static HAUNTER_HANDS: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_hands") });
pub static HAUNTER_IRIS: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eyes") });
pub static HAUNTER_EYE_N: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eyen") });
pub static HAUNTER_EYE_BLINK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_blink") });
pub static HAUNTER_EYE_HALFBLINK1: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_halfblink1") });
pub static HAUNTER_EYE_ATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_attack") });
pub static HAUNTER_EYE_CAPTURE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_capture") });
pub static HAUNTER_EYE_OUCH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_ouch") });
pub static HAUNTER_EYE_DOWN: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_down") });
pub static HAUNTER_EYE_HEAVYATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_eye_heavyattack") });

pub static GENGAR_BODY: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_body") });
pub static GENGAR_TONGUE_LONG: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_tongue_long") });
pub static GENGAR_TONGUE_NORMAL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_tongue_normal") });
pub static GENGAR_IRIS: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eyes") });
pub static GENGAR_EYE_N: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eyen") });
pub static GENGAR_EYE_BLINK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_blink") });
pub static GENGAR_EYE_HALFBLINK1: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_halfblink1") });
pub static GENGAR_EYE_ATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_attack") });
pub static GENGAR_EYE_CAPTURE: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_capture") });
pub static GENGAR_EYE_OUCH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_ouch") });
pub static GENGAR_EYE_DOWN: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_down") });
pub static GENGAR_EYE_HEAVYATTACK: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_eye_heavyattack") });

pub static MEGA_GENGAR_BODY: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("mega_gengar") });
pub static GIGA_GENGAR_BODY: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("giga_gengar") });

pub static GASTLY_EYE_EXPRESSIONS: Lazy<[Hash40; 8]> = Lazy::new(|| [ *GASTLY_EYE_N, *GASTLY_EYE_BLINK, *GASTLY_EYE_HALFBLINK1, *GASTLY_EYE_ATTACK, *GASTLY_EYE_CAPTURE, *GASTLY_EYE_OUCH, *GASTLY_EYE_DOWN, *GASTLY_EYE_HEAVYATTACK ]);
pub static HAUNTER_EYELID_EXPRESSIONS: Lazy<[Hash40; 8]> = Lazy::new(|| [ *HAUNTER_EYE_N, *HAUNTER_EYE_BLINK, *HAUNTER_EYE_HALFBLINK1, *HAUNTER_EYE_ATTACK, *HAUNTER_EYE_CAPTURE, *HAUNTER_EYE_OUCH, *HAUNTER_EYE_DOWN, *HAUNTER_EYE_HEAVYATTACK ]);
pub static GENGAR_EYELID_EXPRESSIONS: Lazy<[Hash40; 8]> = Lazy::new(|| [ *GENGAR_EYE_N, *GENGAR_EYE_BLINK, *GENGAR_EYE_HALFBLINK1, *GENGAR_EYE_ATTACK, *GENGAR_EYE_CAPTURE, *GENGAR_EYE_OUCH, *GENGAR_EYE_DOWN, *GENGAR_EYE_HEAVYATTACK ]);
pub static PURIN_VANILLA_EYES_TO_HIDE: Lazy<[Hash40; 8]> = Lazy::new(|| [ *PURIN_VANILLA_EYE_N, *PURIN_VANILLA_EYE_BLINK, *PURIN_VANILLA_EYE_HALFBLINK1, *PURIN_VANILLA_EYE_ATTACK, *PURIN_VANILLA_EYE_CAPTURE, *PURIN_VANILLA_EYE_OUCH, *PURIN_VANILLA_EYE_DOWN, *PURIN_VANILLA_EYE_HEAVYATTACK ]);

// Shadow Floor meshes (squat_wait animation)
pub static GASTLY_FLOORSHADOW: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_floorshadow") });
pub static HAUNTER_FLOORSHADOW: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_floorshadow") });
pub static GENGAR_FLOORSHADOW: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_floorshadow") });

// Shadow Ball meshes (special_n_hold animations)
pub static GASTLY_SHADOWBALL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gastly_shadowball") });
pub static HAUNTER_SHADOWBALL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("haunter_shadowball") });
pub static GENGAR_SHADOWBALL: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_shadowball") });

// Shadowball delay
pub const SHADOWBALL_MESH_DELAY_FRAMES_GROUND: i32 = 14; // Frame 14 for ground
pub const SHADOWBALL_MESH_DELAY_FRAMES_AIR: i32 = 21;    // Frame 21 for air

// Official Purin Neutral Special Constants
pub const PURIN_SPECIAL_N_HOLD: i32 = 0x1E1;
pub const PURIN_SPECIAL_N_HOLD_MAX: i32 = 0x1E2;
pub const PURIN_SPECIAL_N_ROLL: i32 = 0x1E3;
pub const PURIN_SPECIAL_N_ROLL_AIR: i32 = 0x1E4;
pub const PURIN_SPECIAL_N_TURN: i32 = 0x1E5;
pub const PURIN_SPECIAL_N_END: i32 = 0x1E6;

// Official Purin Neutral Special Work Module Constants
pub const PURIN_WORK_FLOAT_CHARGE_COUNT: i32 = 0x100000C;
pub const PURIN_WORK_INT_HOLD: i32 = 0x11000005;
pub const PURIN_WORK_INT_HOLD_MAX: i32 = 0x11000006;
pub const PURIN_WORK_INT_ROLL_AIR: i32 = 0x11000008;
pub const PURIN_FLAG_MAX_FLAG: i32 = 0x2100000E;
pub const PURIN_FLAG_ATTACK_HIT: i32 = 0x2100000C;

// Neutral Special Mewtwo Effects
pub const FIGHTER_PURIN_STATUS_KIND_SPECIAL_N_HIT_END: i32 = 0x1E7;
pub const MEWTWO_SHADOWBALL_BOMB_DELAY_FRAMES: i32 = 25;

// Ghost mesh (appeal_lw animations - all stages)
pub static GHOST: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("ghost") });

// Gengar Run mesh (run animation - Gengar only)
pub static GENGAR_RUN: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("gengar_run") });

// Animation hashes for detection
pub static SQUAT_WAIT_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("squat_wait") });
pub static SPECIAL_N_HOLD_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("special_n_hold") });
pub static SPECIAL_N_HOLD_MAX_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("special_n_hold_max") });
pub static SPECIAL_AIR_N_HOLD_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("special_air_n_hold") });
pub static SPECIAL_AIR_N_HOLD_MAX_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("special_air_n_hold_max") });
pub static APPEAL_LW_L_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("appeal_lw_l") });
pub static APPEAL_LW_R_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("appeal_lw_r") });
pub static RUN_MOTION: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("run") });

// --- AttackModule Debug and Detection Constants ---
pub const ATTACKMODULE_DEBUG_LOGGING: bool = true; // Set to false to disable debug logs
pub const ATTACKMODULE_DETECTION_FREQUENCY: i32 = 1; // Check every N frames (1 = every frame)
pub const SHADOWBALL_HITBOX_IDS_TO_CHECK: [i32; 3] = [0, 1, 2]; // Attack IDs to check
pub const ATTACKMODULE_FALLBACK_THRESHOLD: i32 = 5; // Frames to wait before falling back to frame detection

// --- Gameplay Constants ---

// Work ID constants for Gastly aura system
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_GASTLY_AURA_FRAME: i32 = 0x60;
pub const FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE: i32 = 0x200000f6;

pub const GASTLY_EVO_DMG_RECEIVED_THRESHOLD: f32 = 35.0;
pub const HAUNTER_EVO_DMG_RECEIVED_THRESHOLD: f32 = 50.0;

pub const GASTLY_EVO_HITS_THRESHOLD: i32 = 10;  // 10 hits to evolve Gastly → Haunter
pub const HAUNTER_EVO_HITS_THRESHOLD: i32 = 15; // 15 hits to evolve Haunter → Gengar

pub const EVOLUTION_ANIMATION_TOTAL_FRAMES: i32 = 240;
pub const EVO_CANCEL_PENALTY_PERCENT: f32 = 50.0; 
pub const MAX_TAUNT_SEQUENCE_LEN: usize = 2;
pub const DEBUG_TAUNT_COOLDOWN: i32 = 30;
pub const DOUBLE_PRESS_WINDOW: i32 = 30; 
pub const ICON_ANIMATION_DURATION: i32 = 40; 
pub const ICON_VERTICAL_OFFSET: f32 = 1.5;   
pub const ICON_Y_POSITION_ABOVE_PLAYER: f32 = 8.0; 
pub const LINKING_CORD_VISUAL_ICON_DURATION: i32 = 150 + ICON_ANIMATION_DURATION;
pub const MANUAL_EVO_HAUNTER_ICON_DURATION: i32 = 120 + ICON_ANIMATION_DURATION; 
pub const EVERSTONE_ICON_DURATION: i32 = 150 + ICON_ANIMATION_DURATION; 
pub const EVERSTONE_X_ICON_DURATION: i32 = 150 + ICON_ANIMATION_DURATION;
pub const FS_MODE_ICON_DURATION: i32 = 120 + ICON_ANIMATION_DURATION;
pub const EVO_CANCEL_DOWN_TAUNT_WINDOW: i32 = 60;
pub const EVO_FLASH_R: f32 = 1.8; 
pub const EVO_FLASH_G: f32 = 1.8;
pub const EVO_FLASH_B: f32 = 1.8;
pub const EVO_FLASH_A: f32 = 0.8; 
pub const FINAL_SMASH_START_R_MOTION_HASH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("final_start_r") });
pub const FINAL_SMASH_MOTION_HASH: Lazy<Hash40> = Lazy::new(|| Hash40 { hash: hash40("final") });

pub static DAMAGE_STATUSES_FOR_ICON_CANCEL: Lazy<[i32; 13]> = Lazy::new(|| [
    *FIGHTER_STATUS_KIND_DAMAGE, *FIGHTER_STATUS_KIND_DAMAGE_AIR, *FIGHTER_STATUS_KIND_DAMAGE_FLY,
    *FIGHTER_STATUS_KIND_DAMAGE_FLY_ROLL, *FIGHTER_STATUS_KIND_DAMAGE_FLY_METEOR,
    *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_LR, *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_U,
    *FIGHTER_STATUS_KIND_DAMAGE_FLY_REFLECT_D, *FIGHTER_STATUS_KIND_DOWN_DAMAGE,
    *FIGHTER_STATUS_KIND_SAVING_DAMAGE, *FIGHTER_STATUS_KIND_SAVING_DAMAGE_AIR,
    *FIGHTER_STATUS_KIND_SAVING_DAMAGE_FLY, *FIGHTER_STATUS_KIND_LANDING
]);