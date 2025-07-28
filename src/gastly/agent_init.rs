// src/gastly/agent_init.rs - Enhanced with flash effect cleanup

use smash::lua2cpp::L2CFighterCommon;
use smash::app::{
    lua_bind::{WorkModule, ModelModule, ColorBlendModule, StatusModule, SoundModule},
    utility,
};
use smash::lib::lua_const::*;
use smash::phx::Hash40;
use smash_script::macros;

// Import necessary items from our modules
use crate::gastly::FIGHTER_STATES;
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};
use crate::gastly::visuals::{update_body_and_unique_parts_visibility, set_active_eye_mesh};
use crate::gastly::icon_management::deactivate_all_pos_sensitive_icons;
use crate::gastly::constants::*;

// Initialize all WorkModule flags and timers for Gastly evolution system
unsafe fn initialize_work_module_flags_and_timers(boma: &mut smash::app::BattleObjectModuleAccessor) {
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_GASTLY_AURA_FRAME);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);

    let sound_flags = [
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_FURAFURA_ACTIVE,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE,
    ];

    let sound_timers = [
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_GRAB_BURN_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_FURAFURA_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER,
        FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER,
    ];

    for &flag in &sound_flags {
        WorkModule::set_flag(boma, false, flag);
    }

    for &timer in &sound_timers {
        WorkModule::set_float(boma, 0.0, timer);
    }
}

// Hide all evolution-related meshes (icons, bodies, readiness indicators)
unsafe fn hide_all_evolution_meshes(boma: &mut smash::app::BattleObjectModuleAccessor) {
    let meshes_to_hide = [
        *LINKING_CORD_ICON,
        *EVERSTONE_ICON,
        *EVERSTONE_X_ICON,
        *GENGARITE_ICON,
        *DYNAMAX_ICON,
        *MEGA_GENGAR_BODY,
        *GIGA_GENGAR_BODY,
        *STG1_DMG_T_ICON,
        *STG1_DMG_D_ICON,
        *STG2_DMG_SS_ICON,
        *STG2_DMG_SE_ICON,
    ];

    for &mesh in &meshes_to_hide {
        ModelModule::set_mesh_visibility(boma, mesh, false);
    }
}

// Force complete reset to Gastly stage for marked costume slots
unsafe fn force_gastly_reset_for_marked_costume(boma: &mut smash::app::BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState, color_id: usize) {
    if crate::is_marked_gastly_costume(boma) {
        player_state.full_reset_on_respawn(boma);
        player_state.stage = EvolutionStage::Gastly;
        player_state.evolution_target_stage = EvolutionStage::Gastly;
        player_state.is_evolving = false;
        player_state.evolution_timer = 0;
        player_state.is_in_final_smash_form = false;
        player_state.mega_gengar_form_active = false;
        player_state.giga_gengar_form_active = false;
        
        update_body_and_unique_parts_visibility(boma, EvolutionStage::Gastly);
        set_active_eye_mesh(boma, player_state, None);
    }
}

pub unsafe extern "C" fn agent_reset_gastly_evolution(fighter: &mut L2CFighterCommon) {
    let boma_raw_ptr = fighter.module_accessor;
    if boma_raw_ptr.is_null() { return; }

    let boma = &mut *boma_raw_ptr;
    let entry_id_val = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);

    if utility::get_kind(boma) == *FIGHTER_KIND_PURIN {
        let entry_id_u32 = entry_id_val as u32;
        let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        let instance_key = crate::gastly::get_instance_key(boma);

        let mut states_map_writer = FIGHTER_STATES.write();
        let player_state = states_map_writer.entry(instance_key).or_insert_with(PlayerEvolutionState::new);

        // Clean up dark effects and flash macros on reset
        crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
        macros::COL_NORMAL(fighter);
        crate::gastly::effects::init_gastly_aura_handle(boma);

        // Reset evolution state and progress
        deactivate_all_pos_sensitive_icons(boma, player_state);
        player_state.full_reset_on_respawn(boma);
        
        player_state.damage_received_this_stage = 0.0;
        player_state.hits_landed_this_stage = 0;
        player_state.previous_total_damage = 0.0;
        
        //  Reset evolution penalties during agent reset
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
        player_state.evo_attempt_delay_hits_penalty = 0;
        
        player_state.reset_evo_readiness_icons();

        // Force Gastly stage for marked costumes
        force_gastly_reset_for_marked_costume(boma, player_state, color_id);

        // Reset visual state to Gastly
        ColorBlendModule::cancel_main_color(boma, 0);
        update_body_and_unique_parts_visibility(boma, EvolutionStage::Gastly);
        set_active_eye_mesh(boma, player_state, None);

        // Initialize clean state
        hide_all_evolution_meshes(boma);
        initialize_work_module_flags_and_timers(boma);

        crate::gastly::persist_sfx::init_evolution_sounds(fighter);
    } else {
        // For non-Purin characters, just clean up flash effects
        let entry_id_u32 = entry_id_val as u32;
        crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
        macros::COL_NORMAL(fighter);
    }
}

// Cancel ongoing evolution process and reset evolution state
unsafe fn cancel_evolution_for_entry(player_state: &mut PlayerEvolutionState, check_boma: *mut smash::app::BattleObjectModuleAccessor) {
    player_state.is_evolving = false;
    player_state.linking_cord_active = false;
    player_state.linking_cord_evo_attempt_icon_timer = 0;
    player_state.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
    player_state.evolution_timer = 0;
    player_state.down_taunt_cancel_press_count = 0;
    player_state.last_evolution_confirmation_frame = -1;
    player_state.reset_evo_readiness_icons();
    player_state.evolution_just_cancelled_this_frame = true;
    
    // Stop evolving sounds
    SoundModule::stop_se(check_boma, Hash40::new("evolving"), 0);
    SoundModule::stop_se(check_boma, Hash40::new("evolve_ss"), 0);
}

unsafe extern "C" fn training_mode_reset_handler(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }
    
    let entry_id_val = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let entry_id_u32 = entry_id_val as u32;
    
    // Force clear flash effects for ALL characters during training reset
    crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
    macros::COL_NORMAL(fighter);

    // Check if ANY entry is in results screen (skip entire reset if so)
    let mut any_in_results = false;
    for check_entry_id in 0..8u32 {
        let check_boma = smash::app::sv_battle_object::module_accessor(check_entry_id);
        if !check_boma.is_null() {
            let status = StatusModule::status_kind(check_boma);
            if status == 0x107 { // Results screen status
                any_in_results = true;
                break;
            }
        }
    }
    
    if any_in_results {
        return;
    }

    // Check ALL possible entries for marked slots and reset them
    let mut states_map_writer = crate::gastly::FIGHTER_STATES.write();
    for (instance_key, player_state) in states_map_writer.iter_mut() {
        let check_entry_id = (*instance_key % 8) as u32; // Convert instance_key back to entry_id
        let check_boma = smash::app::sv_battle_object::module_accessor(check_entry_id);
        if !check_boma.is_null() && utility::get_kind(&mut *check_boma) == *FIGHTER_KIND_PURIN {
            let check_color_id = WorkModule::get_int(check_boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
            let is_marked_slot = check_color_id < 256 && crate::MARKED_COLORS[check_color_id];
            
            if is_marked_slot {
                // Cancel any ongoing evolution
                if player_state.is_evolving {
                    cancel_evolution_for_entry(player_state, check_boma);
                }
                
                // Reset hit counts for ALL marked slots during training reset
                player_state.hits_landed_this_stage = 0;
                player_state.damage_received_this_stage = 0.0;
                player_state.previous_total_damage = 0.0;
                
                //  Reset evolution penalties during training reset
                player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
                player_state.evo_attempt_delay_hits_penalty = 0;
                
                // Reset damage tracking to prevent stale healing detection from previous session
                crate::gastly::reset_damage_tracker_for_entry(check_boma);
                crate::gastly::reset_heal_tracker_for_entry(check_boma);
                
                // Only force stage reset if not already Gastly
                if player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
                    player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.is_evolving = false;
                    player_state.evolution_timer = 0;
                    player_state.is_in_final_smash_form = false;
                    player_state.mega_gengar_form_active = false;
                    player_state.giga_gengar_form_active = false;
                    
                    // Force visual update
                    crate::gastly::visuals::update_body_and_unique_parts_visibility(check_boma, crate::gastly::player_state::EvolutionStage::Gastly);
                    crate::gastly::visuals::set_active_eye_mesh(check_boma, player_state, None);
                }
            }
        }
    }
}

pub fn install() {
    smashline::Agent::new("purin")
        .on_start(agent_reset_gastly_evolution)
        .install();
    
    // Also install for all other fighters to handle flash cleanup
    smashline::Agent::new("fighter")
        .on_start(training_mode_reset_handler)
        .install();
}