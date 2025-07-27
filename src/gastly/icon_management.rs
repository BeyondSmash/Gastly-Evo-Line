// src/gastly/icon_management.rs

use smash::app::lua_bind::{ModelModule, PostureModule, ControlModule, StatusModule, SoundModule};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lib::lua_const::*;
use skyline::libc::c_int;

use crate::gastly::constants::*;
use crate::gastly::player_state::PlayerEvolutionState;
use smash::app::FighterUtil;

pub unsafe fn deactivate_readiness_icons_for_everstone(player_state: &mut PlayerEvolutionState) {
    // When everstone/everstone_x activates, hide readiness icons
    player_state.dmg_t_icon_display_timer = 0;
    player_state.dmg_d_icon_display_timer = 0;
    player_state.dmg_ss_icon_display_timer = 0;
    player_state.dmg_se_icon_display_timer = 0;
}

pub unsafe fn enforce_icon_exclusivity(player_state: &mut PlayerEvolutionState, newly_activated_icon_hash_opt: Option<Hash40>) { 
    let mut deactivated_something = false;
    let is_activating_manual_evo_lc = newly_activated_icon_hash_opt.map_or(false, |h| h.hash == LINKING_CORD_ICON.hash);
    let is_activating_everstone = newly_activated_icon_hash_opt.map_or(false, |h| h.hash == EVERSTONE_ICON.hash || h.hash == EVERSTONE_X_ICON.hash);
    //  If activating everstone, hide readiness icons
    if is_activating_everstone {
        deactivate_readiness_icons_for_everstone(player_state);
    }

    if player_state.linking_cord_visual_icon_active && 
       newly_activated_icon_hash_opt.map_or(true, |h| h.hash != LINKING_CORD_ICON.hash) {
        player_state.linking_cord_visual_icon_active = false;
        player_state.linking_cord_visual_icon_timer = 0;
        deactivated_something = true;
    }

    if player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && 
       player_state.linking_cord_evo_attempt_icon_timer > 0 &&
       !is_activating_manual_evo_lc {
        if newly_activated_icon_hash_opt.map_or(true, |h| h.hash != LINKING_CORD_ICON.hash) {
            player_state.linking_cord_evo_attempt_icon_timer = 0; 
            player_state.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
            deactivated_something = true;
        }
    }
    
    if player_state.everstone_icon_active && 
       newly_activated_icon_hash_opt.map_or(true, |h| h.hash != EVERSTONE_ICON.hash) {
        player_state.everstone_icon_active = false;
        player_state.everstone_icon_timer = 0;
        deactivated_something = true;
    }

    if player_state.everstone_x_icon_active && 
       newly_activated_icon_hash_opt.map_or(true, |h| h.hash != EVERSTONE_X_ICON.hash) {
        player_state.everstone_x_icon_active = false;
        player_state.everstone_x_icon_timer = 0;
        deactivated_something = true;
    }

    if player_state.gengarite_icon_display_active && 
       newly_activated_icon_hash_opt.map_or(true, |h| h.hash != GENGARITE_ICON.hash) {
        player_state.gengarite_icon_display_active = false;
        player_state.gengarite_icon_display_timer = 0;
        deactivated_something = true;
    }

    if player_state.dynamax_icon_display_active && 
       newly_activated_icon_hash_opt.map_or(true, |h| h.hash != DYNAMAX_ICON.hash) {
        player_state.dynamax_icon_display_active = false;
        player_state.dynamax_icon_display_timer = 0;
        deactivated_something = true;
    }

    if deactivated_something {
        update_is_any_pos_sensitive_icon_active_flag(player_state);
    }
}

pub fn update_is_any_pos_sensitive_icon_active_flag(player_state: &mut PlayerEvolutionState) { 
    player_state.is_any_pos_sensitive_icon_active =
        player_state.linking_cord_visual_icon_active ||
        (player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0) || 
        player_state.everstone_icon_active || 
        player_state.everstone_x_icon_active ||
        player_state.gengarite_icon_display_active ||
        player_state.dynamax_icon_display_active;

}

// SIMPLIFIED: Just activate the icon timer, no positioning
pub unsafe fn activate_pos_sensitive_icon( 
    _boma: *mut BattleObjectModuleAccessor,  // Keep parameter but don't use it
    player_state: &mut PlayerEvolutionState, 
    current_player_pos_x: f32, 
    current_player_pos_y: f32 
) { 
    player_state.icon_start_pos_x = current_player_pos_x; 
    player_state.icon_start_pos_y = current_player_pos_y;
    
    update_is_any_pos_sensitive_icon_active_flag(player_state);
}

// SIMPLIFIED: Just reset flags, no billboard bone manipulation
pub unsafe fn deactivate_all_pos_sensitive_icons(
    _boma: *mut BattleObjectModuleAccessor,  // Keep parameter but don't use it
    player_state: &mut PlayerEvolutionState
) { 
    if player_state.linking_cord_visual_icon_active {
        player_state.linking_cord_visual_icon_active = false;
        player_state.linking_cord_visual_icon_timer = 0;
    }
    if player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0 { 
        player_state.linking_cord_evo_attempt_icon_timer = 0; 
        player_state.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
    }
    if player_state.everstone_icon_active {
        player_state.everstone_icon_active = false;
        player_state.everstone_icon_timer = 0; 
    }
    if player_state.everstone_x_icon_active {
        player_state.everstone_x_icon_active = false;
        player_state.everstone_x_icon_timer = 0;
    }
    if player_state.gengarite_icon_display_active {
        player_state.gengarite_icon_display_active = false;
        player_state.gengarite_icon_display_timer = 0;
    }
    if player_state.dynamax_icon_display_active {
        player_state.dynamax_icon_display_active = false;
        player_state.dynamax_icon_display_timer = 0;
    }
    
    update_is_any_pos_sensitive_icon_active_flag(player_state);
}

// Updated handle_icon_toggles_and_effects function with UI flash effects
pub unsafe fn handle_icon_toggles_and_effects(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) { 
    let current_status_val = StatusModule::status_kind(boma); 
    let is_guard_status = current_status_val == *FIGHTER_STATUS_KIND_GUARD || current_status_val == *FIGHTER_STATUS_KIND_GUARD_ON;
    let on_ground = StatusModule::situation_kind(boma) == *SITUATION_KIND_GROUND;
    let current_player_pos_x = PostureModule::pos_x(boma); 
    let current_player_pos_y = PostureModule::pos_y(boma);

    let mut button_pressed_this_frame: Option<i32> = None;
    let appeal_sl_cint = *CONTROL_PAD_BUTTON_APPEAL_S_L as c_int;
    let appeal_sr_cint = *CONTROL_PAD_BUTTON_APPEAL_S_R as c_int;
    let appeal_hi_cint = *CONTROL_PAD_BUTTON_APPEAL_HI as c_int;
    let special_button_cint = *CONTROL_PAD_BUTTON_SPECIAL as c_int;

    if ControlModule::check_button_on_trriger(boma, appeal_sl_cint) { button_pressed_this_frame = Some(appeal_sl_cint); }
    else if ControlModule::check_button_on_trriger(boma, appeal_sr_cint) { button_pressed_this_frame = Some(appeal_sr_cint); }
    else if ControlModule::check_button_on_trriger(boma, appeal_hi_cint) { button_pressed_this_frame = Some(appeal_hi_cint); }
    else if ControlModule::check_button_on_trriger(boma, special_button_cint) { button_pressed_this_frame = Some(special_button_cint); }
    
    let is_in_damage_status = DAMAGE_STATUSES_FOR_ICON_CANCEL.iter().any(|&status| status == current_status_val);

    if player_state.is_any_pos_sensitive_icon_active && is_in_damage_status { 
        deactivate_all_pos_sensitive_icons(boma, player_state);
    }
    else if player_state.is_any_pos_sensitive_icon_active { 
        let pos_diff_x = (current_player_pos_x - player_state.icon_start_pos_x).abs();
        let pos_diff_y = (current_player_pos_y - player_state.icon_start_pos_y).abs();

        if pos_diff_x > 0.01 || pos_diff_y > 0.01 { 
            deactivate_all_pos_sensitive_icons(boma, player_state);
        }
    }

    // Handle specific icon logic with UI effects
    if player_state.linking_cord_consumed_everstone_this_frame { 
        enforce_icon_exclusivity(player_state, Some(*EVERSTONE_X_ICON)); 
        player_state.everstone_x_icon_active = true;
        player_state.everstone_x_icon_timer = EVERSTONE_X_ICON_DURATION;
        player_state.everstone_icon_active = false;
        player_state.everstone_icon_timer = 0;
        activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
        
        // UI EFFECT: Flash when Everstone X appears
        FighterUtil::flash_eye_info(boma);
        // Play everstone_x sound
        crate::gastly::persist_sfx::play_everstone_x_sound(boma);
    }

    // Manual Linking Cord for Haunter (Up Taunt x2 while guarding on ground)
    if is_guard_status && on_ground && player_state.stage == crate::gastly::player_state::EvolutionStage::Haunter {
        if !player_state.is_evolving && !player_state.linking_cord_active { 
            if let Some(button_id) = button_pressed_this_frame {
                if button_id == appeal_hi_cint {
                    if player_state.current_frame - player_state.last_up_taunt_input_frame_linking_cord <= DOUBLE_PRESS_WINDOW {
                        player_state.up_taunt_press_count_linking_cord += 1;
                    } else {
                        player_state.up_taunt_press_count_linking_cord = 1;
                    }
                    player_state.last_up_taunt_input_frame_linking_cord = player_state.current_frame;

                    if player_state.up_taunt_press_count_linking_cord >= 2 {
                        player_state.manual_linking_cord_evo_attempted_this_frame = true;
                        player_state.up_taunt_press_count_linking_cord = 0; 
                        if player_state.linking_cord_visual_icon_active { 
                            player_state.linking_cord_visual_icon_active = false;
                            player_state.linking_cord_visual_icon_timer = 0;
                            update_is_any_pos_sensitive_icon_active_flag(player_state);
                        }
                    } else if player_state.up_taunt_press_count_linking_cord == 1 {
                        if !player_state.linking_cord_visual_icon_active && 
                           !(player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0) {
                            enforce_icon_exclusivity(player_state, Some(*LINKING_CORD_ICON));
                            player_state.linking_cord_visual_icon_active = true;
                            player_state.linking_cord_visual_icon_timer = LINKING_CORD_VISUAL_ICON_DURATION; 
                            activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                            
                            // UI EFFECT: Flash when Linking Cord appears
                            FighterUtil::flash_eye_info(boma);
                            // Play linking_cord sound
                            crate::gastly::persist_sfx::play_linking_cord_sound(boma);
                        }
                    }
                }
            }
        }
    }

    if player_state.up_taunt_press_count_linking_cord == 1 &&
       (player_state.current_frame - player_state.last_up_taunt_input_frame_linking_cord > DOUBLE_PRESS_WINDOW) {
        player_state.up_taunt_press_count_linking_cord = 0; 
        if player_state.linking_cord_visual_icon_active {
            player_state.linking_cord_visual_icon_active = false;
            player_state.linking_cord_visual_icon_timer = 0;
            update_is_any_pos_sensitive_icon_active_flag(player_state);
        }
    }

    // Everstone Toggle (Special x2 while guarding on ground for Gastly/Haunter)
    if is_guard_status && on_ground && 
       (player_state.stage == crate::gastly::player_state::EvolutionStage::Gastly || player_state.stage == crate::gastly::player_state::EvolutionStage::Haunter) {
        if let Some(button_id) = button_pressed_this_frame {
            if button_id == special_button_cint {
                if player_state.is_evolving || player_state.linking_cord_active { 
                    if !player_state.everstone_x_icon_active { 
                        enforce_icon_exclusivity(player_state, Some(*EVERSTONE_X_ICON));
                        player_state.everstone_x_icon_active = true;
                        player_state.everstone_x_icon_timer = EVERSTONE_X_ICON_DURATION;
                        activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                        
                        // UI EFFECT: Flash when Everstone X appears during evolution
                        FighterUtil::flash_eye_info(boma);
                        // Play everstone_x sound
                        crate::gastly::persist_sfx::play_everstone_x_sound(boma);
                    }
                } else {
                    if player_state.current_frame - player_state.last_special_press_frame_everstone <= DOUBLE_PRESS_WINDOW {
                        player_state.special_press_count_everstone += 1;
                    } else {
                        player_state.special_press_count_everstone = 1;
                    }
                    player_state.last_special_press_frame_everstone = player_state.current_frame;

                    if player_state.special_press_count_everstone >= 2 {
                        player_state.everstone_effect_active = !player_state.everstone_effect_active;
                        player_state.special_press_count_everstone = 0; 

                        if player_state.everstone_effect_active {
                            enforce_icon_exclusivity(player_state, Some(*EVERSTONE_ICON));
                            player_state.everstone_icon_active = true; 
                            player_state.everstone_icon_timer = EVERSTONE_ICON_DURATION;
                            activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                            
                            // UI EFFECT: Flash when Everstone activates
                            FighterUtil::flash_eye_info(boma);
                            // Play everstone sound
                            crate::gastly::persist_sfx::play_everstone_sound(boma);
                        } else {
                            enforce_icon_exclusivity(player_state, Some(*EVERSTONE_X_ICON));
                            player_state.everstone_icon_active = false; 
                            player_state.everstone_icon_timer = 0;
                            player_state.everstone_x_icon_active = true; 
                            player_state.everstone_x_icon_timer = EVERSTONE_X_ICON_DURATION;
                            activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                            
                            // UI EFFECT: Flash when Everstone deactivates (X appears)
                            FighterUtil::flash_eye_info(boma);
                            // Play everstone_x sound
                            crate::gastly::persist_sfx::play_everstone_x_sound(boma);
                        }
                    }
                }
            }
        }
    }

    if player_state.special_press_count_everstone == 1 &&
       (player_state.current_frame - player_state.last_special_press_frame_everstone > DOUBLE_PRESS_WINDOW) {
        player_state.special_press_count_everstone = 0;
    }
    
    // Gengar Final Smash Mode Selection (Gengarite/Dynamax)
    if player_state.stage == crate::gastly::player_state::EvolutionStage::Gengar && on_ground && is_guard_status {
        if let Some(button_id) = button_pressed_this_frame {
            if button_id == appeal_sl_cint {
            if player_state.current_frame - player_state.last_s_taunt_r_input_frame_dynamax <= DOUBLE_PRESS_WINDOW {
                player_state.s_taunt_r_press_count_dynamax += 1;
            } else {
                player_state.s_taunt_r_press_count_dynamax = 1;
            }
            player_state.last_s_taunt_r_input_frame_dynamax = player_state.current_frame;

            if player_state.s_taunt_r_press_count_dynamax >= 2 {
                enforce_icon_exclusivity(player_state, Some(*GENGARITE_ICON)); 
                player_state.mega_gengar_form_active = true; 
                player_state.giga_gengar_form_active = false; 
                player_state.gengarite_icon_display_active = true;
                player_state.gengarite_icon_display_timer = FS_MODE_ICON_DURATION;
                activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                player_state.s_taunt_r_press_count_dynamax = 0;
                
                // UI EFFECT: Flash when Gengarite appears
                FighterUtil::flash_eye_info(boma);
                // Play gengarite sound
                crate::gastly::persist_sfx::play_gengarite_sound(boma);
            }
        }

            if button_id == appeal_sr_cint {
                if player_state.current_frame - player_state.last_s_taunt_r_input_frame_dynamax <= DOUBLE_PRESS_WINDOW {
                    player_state.s_taunt_r_press_count_dynamax += 1;
                } else {
                    player_state.s_taunt_r_press_count_dynamax = 1;
                }
                player_state.last_s_taunt_r_input_frame_dynamax = player_state.current_frame;

                if player_state.s_taunt_r_press_count_dynamax >= 2 {
                    enforce_icon_exclusivity(player_state, Some(*DYNAMAX_ICON)); 
                    player_state.giga_gengar_form_active = true; 
                    player_state.mega_gengar_form_active = false;
                    player_state.dynamax_icon_display_active = true;
                    player_state.dynamax_icon_display_timer = FS_MODE_ICON_DURATION;
                    activate_pos_sensitive_icon(boma, player_state, current_player_pos_x, current_player_pos_y);
                    player_state.s_taunt_r_press_count_dynamax = 0;
                    
                    // UI EFFECT: Flash when Dynamax appears
                    FighterUtil::flash_eye_info(boma);
                    // Play dynamax sound
                    crate::gastly::persist_sfx::play_dynamax_sound(boma);
                }
            }
        }
    }

    if player_state.s_taunt_r_press_count_dynamax == 1 &&
       (player_state.current_frame - player_state.last_s_taunt_r_input_frame_dynamax > DOUBLE_PRESS_WINDOW) {
        player_state.s_taunt_r_press_count_dynamax = 0;
    }

    if !player_state.gengarite_input_sequence.is_empty() &&
        player_state.current_frame - player_state.last_gengarite_input_frame > DOUBLE_PRESS_WINDOW { 
        player_state.gengarite_input_sequence.clear();
    }

    // Handle icon timers - SIMPLIFIED, no animation check
    if player_state.linking_cord_visual_icon_active && player_state.linking_cord_visual_icon_timer > 0 {
        player_state.linking_cord_visual_icon_timer -= 1;
        if player_state.linking_cord_visual_icon_timer == 0 {
            player_state.linking_cord_visual_icon_active = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state); 
        }
    }

    if player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0 {
        player_state.linking_cord_evo_attempt_icon_timer -=1;
        if player_state.linking_cord_evo_attempt_icon_timer == 0 {
            player_state.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state);
        }
    } 
    else if !player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0 { 
        player_state.linking_cord_evo_attempt_icon_timer -=1;
    }

    if player_state.everstone_icon_active && player_state.everstone_icon_timer > 0 { 
        player_state.everstone_icon_timer -= 1;
        if player_state.everstone_icon_timer == 0 {
            player_state.everstone_icon_active = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state); 
        }
    }

    if player_state.everstone_x_icon_active && player_state.everstone_x_icon_timer > 0 {
        player_state.everstone_x_icon_timer -= 1;
        if player_state.everstone_x_icon_timer == 0 {
            player_state.everstone_x_icon_active = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state);
        }
    }

    if player_state.gengarite_icon_display_active && player_state.gengarite_icon_display_timer > 0 {
        player_state.gengarite_icon_display_timer -= 1;
        if player_state.gengarite_icon_display_timer == 0 {
            player_state.gengarite_icon_display_active = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state); 
        }
    }

    if player_state.dynamax_icon_display_active && player_state.dynamax_icon_display_timer > 0 {
        player_state.dynamax_icon_display_timer -= 1;
        if player_state.dynamax_icon_display_timer == 0 {
            player_state.dynamax_icon_display_active = false;
            update_is_any_pos_sensitive_icon_active_flag(player_state); 
        }
    }
    
    // Update mesh visibility for icons based on player_state flags
    ModelModule::set_mesh_visibility(boma, *LINKING_CORD_ICON,
        player_state.linking_cord_visual_icon_active || 
        (player_state.linking_cord_evo_attempt_icon_timer > 0)
    );
    ModelModule::set_mesh_visibility(boma, *EVERSTONE_ICON,
        player_state.everstone_icon_active && player_state.everstone_effect_active
    );
    ModelModule::set_mesh_visibility(boma, *EVERSTONE_X_ICON, 
        player_state.everstone_x_icon_active 
    );
    ModelModule::set_mesh_visibility(boma, *GENGARITE_ICON, 
        player_state.gengarite_icon_display_active 
    );
    ModelModule::set_mesh_visibility(boma, *DYNAMAX_ICON, 
        player_state.dynamax_icon_display_active 
    );
} 