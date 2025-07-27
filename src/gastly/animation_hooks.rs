// src/gastly/animation_hooks.rs

use smash::app::lua_bind::{StatusModule, WorkModule, MotionModule, ModelModule, VisibilityModule};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lib::lua_const::*;
use skyline::libc::c_uint;

use crate::gastly::constants::*;
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};

// Motion-based expression detection for specific animations
unsafe fn detect_motion_based_expression(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState
) -> Option<Hash40> {
    let status = StatusModule::status_kind(boma);
    let motion_frame = MotionModule::frame(boma);
    
    // Rest animation - eyes close during sleep
    if status == *FIGHTER_STATUS_KIND_SPECIAL_LW {
        if motion_frame >= 10.0 && motion_frame <= 60.0 {
            return Some(match player_state.stage {
                EvolutionStage::Gastly => *GASTLY_EYE_BLINK,
                EvolutionStage::Haunter => *HAUNTER_EYE_BLINK,
                EvolutionStage::Gengar => *GENGAR_EYE_BLINK,
            });
        }
    }
    
    // Attack animations
    if status >= *FIGHTER_STATUS_KIND_ATTACK && status <= *FIGHTER_STATUS_KIND_ATTACK_LW4 {
        return Some(match player_state.stage {
            EvolutionStage::Gastly => *GASTLY_EYE_ATTACK,
            EvolutionStage::Haunter => *HAUNTER_EYE_ATTACK,
            EvolutionStage::Gengar => *GENGAR_EYE_ATTACK,
        });
    }
    
    None
}

// Vanilla expression polling - checks every 2 frames for high accuracy
unsafe fn detect_vanilla_expression_change(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState
) {
    // Check every 2 frames for high accuracy with good performance
    if player_state.current_frame % 2 != 0 {
        return;
    }
    
    for vanilla_eye in PURIN_VANILLA_EYES_TO_HIDE.iter() {
        if VisibilityModule::is_visible_mesh(boma, vanilla_eye.hash as c_uint) {
            let custom_expression = map_vanilla_to_custom_expression_direct(*vanilla_eye, player_state.stage);
            if let Some(custom_expr) = custom_expression {
                if player_state.last_vanilla_expression.hash != custom_expr.hash {
                    player_state.last_vanilla_expression = custom_expr;
                    player_state.vanilla_expression_changed = true;
                    return;
                }
            }
        }
    }
}

// Direct mapping from vanilla mesh hash to custom expression
fn map_vanilla_to_custom_expression_direct(vanilla_mesh: Hash40, stage: EvolutionStage) -> Option<Hash40> {
    let expression_type = if vanilla_mesh.hash == PURIN_VANILLA_EYE_N.hash {
        "normal"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_BLINK.hash {
        "blink"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_HALFBLINK1.hash {
        "halfblink"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_ATTACK.hash {
        "attack"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_CAPTURE.hash {
        "capture"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_OUCH.hash {
        "ouch"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_DOWN.hash {
        "down"
    } else if vanilla_mesh.hash == PURIN_VANILLA_EYE_HEAVYATTACK.hash {
        "heavyattack"
    } else {
        return None;
    };

    match (expression_type, stage) {
        ("normal", EvolutionStage::Gastly) => Some(*GASTLY_EYE_N),
        ("blink", EvolutionStage::Gastly) => Some(*GASTLY_EYE_BLINK),
        ("halfblink", EvolutionStage::Gastly) => Some(*GASTLY_EYE_HALFBLINK1),
        ("attack", EvolutionStage::Gastly) => Some(*GASTLY_EYE_ATTACK),
        ("capture", EvolutionStage::Gastly) => Some(*GASTLY_EYE_CAPTURE),
        ("ouch", EvolutionStage::Gastly) => Some(*GASTLY_EYE_OUCH),
        ("down", EvolutionStage::Gastly) => Some(*GASTLY_EYE_DOWN),
        ("heavyattack", EvolutionStage::Gastly) => Some(*GASTLY_EYE_HEAVYATTACK),

        ("normal", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_N),
        ("blink", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_BLINK),
        ("halfblink", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_HALFBLINK1),
        ("attack", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_ATTACK),
        ("capture", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_CAPTURE),
        ("ouch", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_OUCH),
        ("down", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_DOWN),
        ("heavyattack", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_HEAVYATTACK),

        ("normal", EvolutionStage::Gengar) => Some(*GENGAR_EYE_N),
        ("blink", EvolutionStage::Gengar) => Some(*GENGAR_EYE_BLINK),
        ("halfblink", EvolutionStage::Gengar) => Some(*GENGAR_EYE_HALFBLINK1),
        ("attack", EvolutionStage::Gengar) => Some(*GENGAR_EYE_ATTACK),
        ("capture", EvolutionStage::Gengar) => Some(*GENGAR_EYE_CAPTURE),
        ("ouch", EvolutionStage::Gengar) => Some(*GENGAR_EYE_OUCH),
        ("down", EvolutionStage::Gengar) => Some(*GENGAR_EYE_DOWN),
        ("heavyattack", EvolutionStage::Gengar) => Some(*GENGAR_EYE_HEAVYATTACK),

        _ => None,
    }
}

// Main detection function called from player_state.rs
pub unsafe fn detect_expression_from_game_state(
    boma: *mut BattleObjectModuleAccessor, 
    player_state: &mut PlayerEvolutionState
) -> Option<Hash40> {
    if !player_state.vanilla_expression_tracking {
        return None;
    }

    // First try motion-based detection for specific animations
    if let Some(motion_expression) = detect_motion_based_expression(boma, player_state) {
        return Some(motion_expression);
    }

    // If we have a tracked expression from previous detection, use it
    if player_state.vanilla_expression_changed {
        player_state.vanilla_expression_changed = false;
        return Some(player_state.last_vanilla_expression);
    }

    // Fallback to polling vanilla expressions
    detect_vanilla_expression_change(boma, player_state);
    if player_state.last_vanilla_expression.hash != 0 {
        return Some(player_state.last_vanilla_expression);
    }

    None
}

pub fn install_animation_hooks() {
    } 