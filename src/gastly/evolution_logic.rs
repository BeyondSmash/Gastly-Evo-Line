// src/gastly/evolution_logic.rs - Updated animation advancement function

use smash::app::lua_bind::{StatusModule, ControlModule, MotionModule, ModelModule, PostureModule, EffectModule, SoundModule}; 
use smash::app::BattleObjectModuleAccessor;
use smash::phx::{Hash40, Vector3f};
use smash::lua2cpp::L2CFighterCommon;
use smash::lib::lua_const::*; 
use skyline::libc::c_int; 
use smash_script::macros; 

// Import constants
use crate::gastly::constants::*;
// Import PlayerEvolutionState and EvolutionStage enum
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};
// Import visual functions needed for animation
use crate::gastly::visuals::{update_body_and_unique_parts_visibility, set_active_eye_mesh};

// This function is called every frame to check for evolution triggers or cancellations.
pub unsafe fn handle_evolution_process(fighter: &mut L2CFighterCommon, player_state: &mut PlayerEvolutionState) { 
    let boma = fighter.module_accessor;
    let current_status_val = StatusModule::status_kind(boma);
    let is_guard_status = current_status_val == *FIGHTER_STATUS_KIND_GUARD || current_status_val == *FIGHTER_STATUS_KIND_GUARD_ON;
    let on_ground = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    let in_air = StatusModule::situation_kind(boma) == *SITUATION_KIND_AIR;

    // Check for evolution cancellation input if currently evolving
    if player_state.is_evolving {
        let mut cancelled_by_input = false;
        // Down Taunt x2 to cancel (during guard on ground or anytime in air)
        if ControlModule::check_button_on_trriger(boma, *CONTROL_PAD_BUTTON_APPEAL_LW as c_int) {
            if (on_ground && is_guard_status) || in_air { 
                if player_state.current_frame - player_state.last_down_taunt_cancel_input_frame <= EVO_CANCEL_DOWN_TAUNT_WINDOW {
                    player_state.down_taunt_cancel_press_count += 1;
                } else {
                    player_state.down_taunt_cancel_press_count = 1;
                }
                player_state.last_down_taunt_cancel_input_frame = player_state.current_frame;

                if player_state.down_taunt_cancel_press_count >= 2 {
                    cancelled_by_input = true;
                }
            }
        }
        // Reset cancel input count if window expires
        if player_state.down_taunt_cancel_press_count == 1 &&
           player_state.current_frame - player_state.last_down_taunt_cancel_input_frame > EVO_CANCEL_DOWN_TAUNT_WINDOW {
            player_state.down_taunt_cancel_press_count = 0;
        }

        if cancelled_by_input {
            player_state.cancel_evolution(fighter);
        }
        return; // If evolving (or just cancelled), don't check for new evolution triggers this frame
    }
    
    // Check for manual evolution trigger for Haunter to Gengar
    if player_state.stage == EvolutionStage::Haunter && on_ground && is_guard_status {
        if player_state.manual_linking_cord_evo_attempted_this_frame { 
            // If Everstone was active, attempting manual evolution consumes it
            if player_state.everstone_effect_active {
                player_state.everstone_effect_active = false; 
                player_state.linking_cord_consumed_everstone_this_frame = true;
            }

            // Try to start evolution if Everstone is now off (either was off or just consumed)
            if !player_state.everstone_effect_active { 
                if !player_state.is_evolving && !player_state.linking_cord_active {
                    player_state.start_evolution_process(EvolutionStage::Gengar, fighter, true);
                }
            }
        }
    }
    
    // If already evolving (e.g., manual trigger started it) or Everstone is active, don't check auto-evo
    if player_state.is_evolving || player_state.everstone_effect_active { return; } 

    // NEW: Check if readiness icons are currently displaying - if so, delay auto-evolution
    let readiness_icons_active = player_state.dmg_t_icon_display_timer > 0 ||
                                player_state.dmg_d_icon_display_timer > 0 ||
                                player_state.dmg_ss_icon_display_timer > 0 ||
                                player_state.dmg_se_icon_display_timer > 0;
    
    if readiness_icons_active {
        return; // Don't auto-evolve while readiness icons are showing
    }

    // Check for automatic evolution based on damage thresholds
    let next_stage_candidate = match player_state.stage {
        EvolutionStage::Gastly => Some(EvolutionStage::Haunter),
        EvolutionStage::Haunter => Some(EvolutionStage::Gengar),
        _ => None, // Gengar cannot auto-evolve further
    };

    if let Some(target_stage) = next_stage_candidate {
        // start_evolution_process will check damage thresholds internally
        player_state.start_evolution_process(target_stage, fighter, false);
    }
}

// FIXED: Evolution animation with evolving mesh system instead of flash
pub unsafe fn advance_evolution_animation(fighter: &mut L2CFighterCommon, player_state: &mut PlayerEvolutionState) { 
    let boma = fighter.module_accessor;
    player_state.evolution_timer += 1;
    
    // Check if player is moving during evolution
    let current_pos_x = PostureModule::pos_x(boma);
    let current_pos_y = PostureModule::pos_y(boma);
    
    // Store initial position if this is the first frame of evolution
    if player_state.evolution_timer == 1 {
        player_state.evolution_start_pos_x = current_pos_x;
        player_state.evolution_start_pos_y = current_pos_y;
        
        // Initialize status tracking
        player_state.last_status_during_evolution = StatusModule::status_kind(boma);
        
        // IMPORTANT: Remove any lingering flash effects from previous evolution attempts
        macros::COL_NORMAL(fighter);
        
        println!("[EVOLUTION] Started evolving mesh animation - no more flash!");
    }

    // ganon_final_transform effect at 3/4 of evolution (180/240 frames)
    if player_state.evolution_timer == 180 {
        let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
        
        let handle = EffectModule::req_follow(
            boma,
            Hash40::new("ganon_final_transform"),
            Hash40::new("body"),
            &position_offset,
            &rotation_vector,
            0.2,
            true, 0x40000, 0, -1, 0, 0, false, false
        ) as u32;
        
        if handle != u64::MAX as u32 && handle != 0 {
            EffectModule::set_rate(boma, handle, 0.4);
            println!("[EVOLUTION] Spawned ganon transform effect");
        }
    }
    
    // Check if player has moved significantly from evolution start position
    let pos_diff_x = (current_pos_x - player_state.evolution_start_pos_x).abs();
    let pos_diff_y = (current_pos_y - player_state.evolution_start_pos_y).abs();
    let is_moving = pos_diff_x > 0.5 || pos_diff_y > 0.5;
    
    // Hide linking cord icon if player is moving during evolution
    if is_moving && player_state.linking_cord_evo_attempt_icon_timer > 0 {
        ModelModule::set_mesh_visibility(boma, *LINKING_CORD_ICON, false);
    } else if player_state.linking_cord_evo_attempt_icon_timer > 0 {
        ModelModule::set_mesh_visibility(boma, *LINKING_CORD_ICON, true);
    }
    
    // CRITICAL: The evolving mesh system now handles all visual changes!
    // The set_active_eye_mesh function will detect player_state.is_evolving = true
    // and automatically show the appropriate evolving meshes based on:
    // - Current animation (squat_wait -> evolving floor shadow)
    // - Shadowball status (hold/hold_max with delay -> evolving shadowball)  
    // - Default case (main evolving mesh)
    
    // Keep normal blinking for current stage during evolution
    player_state.blink_timer -= 1;
    if player_state.blink_timer <= 0 {
        player_state.advance_blink_phase();
    }
    
    // The visual system will automatically show:
    // 1. Appropriate evolving mesh (main/floorshadow/shadowball)
    // 2. Current stage eye expressions with normal blinking
    // 3. Iris for Haunter/Gengar stages
    // No flash overlay needed - the evolving material does the visual work!
    
    // Just call set_active_eye_mesh - it handles everything now
    set_active_eye_mesh(boma, player_state, None);

    // Check if evolution animation duration is complete
    if player_state.evolution_timer >= EVOLUTION_ANIMATION_TOTAL_FRAMES {
        println!("[EVOLUTION] Animation complete - confirming evolution and clearing evolving meshes!");
        player_state.confirm_evolution(fighter);
    }
}