// src/gastly/agent_init.rs - Enhanced with flash effect cleanup

use smash::lua2cpp::L2CFighterCommon;
use smash::app::{
    lua_bind::{WorkModule, ModelModule, ColorBlendModule, StatusModule},
    utility,
};
use smash::app::BattleObjectModuleAccessor;
use smash::lib::lua_const::*;
use smash_script::macros;

// Import necessary items from our modules
use crate::gastly::FIGHTER_STATES;
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};
use crate::gastly::visuals::{update_body_and_unique_parts_visibility, set_active_eye_mesh};
use crate::gastly::icon_management::deactivate_all_pos_sensitive_icons;
use crate::gastly::constants::*; // Ensure all constants are imported

pub unsafe extern "C" fn agent_reset_gastly_evolution(fighter: &mut L2CFighterCommon) {
    let boma_raw_ptr = fighter.module_accessor;
    if boma_raw_ptr.is_null() { return; }

    let boma = &mut *boma_raw_ptr;
    // LOG ALL CALLS for debugging
    let fighter_kind_val: i32 = utility::get_kind(boma);
    let entry_id_val = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);

    println!("[AGENT_INIT DEBUG] Called for kind: {}, entry: {}, color: {}", fighter_kind_val, entry_id_val, color_id);

    if utility::get_kind(boma) == *FIGHTER_KIND_PURIN {
        let entry_id_val = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        let entry_id_u32 = entry_id_val as u32;

        let mut states_map_writer = FIGHTER_STATES.write();

        let _is_new_state = !states_map_writer.contains_key(&entry_id_u32);
        let player_state = states_map_writer.entry(entry_id_u32).or_insert_with(PlayerEvolutionState::new);

        println!("[AGENT_INIT] Processing Purin entry {} color c{:02} - Current stage: {:?}", entry_id_u32, color_id, player_state.stage);

        // Clean up dark effects AND flash macros on reset
        crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
        
        // Force clear any lingering flash macros immediately
        macros::COL_NORMAL(fighter);
        println!("[AGENT_INIT] Forced COL_NORMAL for entry {} on training reset", entry_id_u32);

        // Initialize Gastly aura handle - this is the key addition
        crate::gastly::effects::init_gastly_aura_handle(boma);

        // Deactivate any icons and reset billboard bone first
        deactivate_all_pos_sensitive_icons(boma, player_state);

        player_state.full_reset_on_respawn(boma); //This will call reset_evo_readiness_icons internally

        // Force reset evolution progress for new sessions
        player_state.damage_received_this_stage = 0.0;
        player_state.hits_landed_this_stage = 0;
        player_state.previous_total_damage = 0.0;
        player_state.reset_evo_readiness_icons();

        // CRITICAL: Always force Gastly stage for marked costumes in agent reset
        let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
            // Force complete reset to Gastly regardless of previous state
            player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
            player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
            player_state.is_evolving = false;
            player_state.evolution_timer = 0;
            player_state.is_in_final_smash_form = false;
            player_state.mega_gengar_form_active = false;
            player_state.giga_gengar_form_active = false;
            
            println!("[AGENT_INIT] ★ FORCED GASTLY RESET IN AGENT_INIT ★ for marked costume c{:02}", color_id);
            
            // Force visual update immediately
            update_body_and_unique_parts_visibility(boma, EvolutionStage::Gastly);
            set_active_eye_mesh(boma, player_state, None);
        }

        // CRITICAL: Force Gastly stage for marked costumes on agent reset
        let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
            player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
            player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
            println!("[AGENT_INIT] Forced Gastly stage for marked costume c{:02}", color_id);
        }

        // CRITICAL: For marked costumes, ALWAYS force complete reset regardless of previous state
        if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
            // Completely recreate the player state for marked costumes
            player_state.full_reset_on_respawn(boma);
            player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
            player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
            player_state.is_evolving = false;
            player_state.evolution_timer = 0;
            player_state.is_in_final_smash_form = false;
            player_state.mega_gengar_form_active = false;
            player_state.giga_gengar_form_active = false;
            
            println!("[AGENT_INIT] ★ FORCED COMPLETE GASTLY RESET ★ for marked costume c{:02}", color_id);
        }

        player_state.previous_total_damage = 0.0;

        ColorBlendModule::cancel_main_color(boma, 0);
        update_body_and_unique_parts_visibility(boma, EvolutionStage::Gastly);
        set_active_eye_mesh(boma, player_state, None);

        // Explicitly hide icon meshes as deactivate_all_pos_sensitive_icons only sets flags
        // and handle_icon_toggles_and_effects won't run until the main loop.
        // Also hide readiness icons.
        ModelModule::set_mesh_visibility(boma, *LINKING_CORD_ICON, false);
        ModelModule::set_mesh_visibility(boma, *EVERSTONE_ICON, false);
        ModelModule::set_mesh_visibility(boma, *EVERSTONE_X_ICON, false);
        ModelModule::set_mesh_visibility(boma, *GENGARITE_ICON, false);
        ModelModule::set_mesh_visibility(boma, *DYNAMAX_ICON, false);
        ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);

        // Hide readiness icons
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);

        // Initialize Gastly aura work module values here instead of separate function
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_GASTLY_AURA_FRAME);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);

        // Initialize all looping sound flags
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_FURAFURA_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
        
        // Initialize all looping sound timers
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_GRAB_BURN_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_FURAFURA_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);

        println!("[AGENT_INIT] Completed initialization for Purin entry {}", entry_id_u32);

        crate::gastly::persist_sfx::init_evolution_sounds(fighter);
    } else {
        // For non-Purin characters
        let entry_id_u32 = entry_id_val as u32;
        crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
        macros::COL_NORMAL(fighter);
        println!("[AGENT_INIT] Forced COL_NORMAL for non-Purin entry {} on training reset", entry_id_u32);
    }
}

unsafe extern "C" fn training_mode_reset_handler(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }
    
    let entry_id_val = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let entry_id_u32 = entry_id_val as u32;
    
    // Force clear flash effects for ALL characters during training reset
    crate::gastly::darkfx::cleanup_dark_effects_on_death(entry_id_u32);
    macros::COL_NORMAL(fighter);
    
    println!("[TRAINING_RESET] Cleaned up flash effects for entry {}", entry_id_u32);

    // Check if ANY entry is in results screen (skip entire reset if so)
    let mut any_in_results = false;
    for check_entry_id in 0..8u32 {
        let check_boma = smash::app::sv_battle_object::module_accessor(check_entry_id);
        if !check_boma.is_null() {
            let status = StatusModule::status_kind(check_boma);
            if status == 0x107 { // Only the specific results status
                any_in_results = true;
                break;
            }
        }
    }
    
    if any_in_results {
        println!("[TRAINING_RESET] Skipping reset - results screen active");
        return;
    }

    // Check ALL possible entries for marked slots (like the old working version)
    let mut states_map_writer = crate::gastly::FIGHTER_STATES.write();
    for (check_entry_id, player_state) in states_map_writer.iter_mut() {
        // Get the boma for this specific entry to check its color
        let check_boma = smash::app::sv_battle_object::module_accessor(*check_entry_id);
        if !check_boma.is_null() && utility::get_kind(&mut *check_boma) == *FIGHTER_KIND_PURIN {
            let check_color_id = WorkModule::get_int(check_boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
            let is_marked_slot = if check_color_id < 256 {
                unsafe { crate::MARKED_COLORS[check_color_id] }
            } else {
                false
            };
            
            if is_marked_slot {
                // Check if player is currently evolving and cancel it first
                if player_state.is_evolving {
                    println!("[TRAINING_RESET] Player {} is evolving - cancelling evolution", check_entry_id);
                    
                    // Manually cancel evolution without needing L2CFighterCommon
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
                    use smash::app::lua_bind::SoundModule;
                    use smash::phx::Hash40;
                    SoundModule::stop_se(check_boma, Hash40::new("evolving"), 0);
                    SoundModule::stop_se(check_boma, Hash40::new("evolve_ss"), 0);
                    
                    println!("[TRAINING_RESET] Evolution cancelled for entry {}", check_entry_id);
                }
                
                // Reset hit counts for ALL marked slots during training reset
                let old_hits = player_state.hits_landed_this_stage;
                player_state.hits_landed_this_stage = 0;
                player_state.damage_received_this_stage = 0.0;
                player_state.previous_total_damage = 0.0;
                
                println!("[TRAINING_RESET] Reset hit count from {} to 0 for entry {} c{:02}", 
                        old_hits, check_entry_id, check_color_id);
                
                // Only force stage reset if not already Gastly
                if player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
                    println!("[TRAINING_RESET] ★ FORCING GASTLY RESET ★ Entry {}, {:?} -> Gastly for c{:02}", 
                            check_entry_id, player_state.stage, check_color_id);
                    
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
    
    // NEW: Also install for all other fighters to handle flash cleanup
    smashline::Agent::new("fighter")
        .on_start(training_mode_reset_handler)
        .install();
}