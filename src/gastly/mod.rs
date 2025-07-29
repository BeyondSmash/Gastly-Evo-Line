// src/gastly/mod.rs - Fixed version with proper down special blink handling

// Standard library and crate imports
use std::collections::HashMap;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

// Smash and Skyline imports
use skyline::hook;
use smash::lib::lua_const::*;
use smash::app::{
    lua_bind::{
        WorkModule, StatusModule, ControlModule, ModelModule,
        DamageModule, MotionModule, EffectModule, FighterManager,
        SoundModule, AttackModule
    },
    utility,
    BattleObjectModuleAccessor,
    FighterUtil,
};
use smash::phx::{Hash40 as PhxHash40, Vector3f};
use smash::lua2cpp::{L2CFighterCommon};
use smashline::*;
use smash_script::macros;

// Declare our submodules
pub mod constants;
pub mod player_state;
pub mod visuals;
mod icon_management;
pub mod evolution_logic;
pub mod agent_init;
mod random_module;
pub mod animation_hooks;
pub mod effects;
pub mod acmd;
pub mod darkfx;
pub mod sounds;
pub mod acmdsound;
mod persist_sfx;
pub mod attack_voices;
mod ui_management;

// Use items from our submodules
use crate::gastly::constants::*;
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage, BlinkPhase};
use crate::gastly::visuals::{update_body_and_unique_parts_visibility_with_enforcement, update_body_and_unique_parts_visibility, set_active_eye_mesh, handle_final_smash_model_swap, hide_all_animation_specific_meshes};
use crate::gastly::icon_management::{handle_icon_toggles_and_effects, deactivate_all_pos_sensitive_icons};
use crate::gastly::evolution_logic::{handle_evolution_process, advance_evolution_animation};
use crate::gastly::effects::{handle_gastly_effects, GASTLY_AURA_HANDLE_WORK_ID};
use crate::gastly::ui_management::{handle_ui_management, reset_ui_state_on_death, track_cry_sound_playback};


// Death cleanup function
unsafe fn cleanup_all_evolution_sounds_on_death(boma: *mut BattleObjectModuleAccessor) {
    static mut EVOLVING_SOUND_HANDLE: [u32; 256] = [0; 256];
    static mut EVOLVE_SS_SOUND_HANDLE: [u32; 256] = [0; 256];
    static mut SHADOWBALL_CHARGE_HANDLE: [u32; 256] = [0; 256];
    static mut G_GRAB_BURN_HANDLE: [u32; 256] = [0; 256];     
    static mut MEGASYMBOL_HANDLE: [u32; 256] = [0; 256];    
    
    let instance_key = get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    // Stop all sounds by handle AND by name (double safety)
    if EVOLVING_SOUND_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, EVOLVING_SOUND_HANDLE[instance_key] as i32, 0);
        EVOLVING_SOUND_HANDLE[instance_key] = 0;
    }
    if EVOLVE_SS_SOUND_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, EVOLVE_SS_SOUND_HANDLE[instance_key] as i32, 0);
        EVOLVE_SS_SOUND_HANDLE[instance_key] = 0;
    }
    if SHADOWBALL_CHARGE_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, SHADOWBALL_CHARGE_HANDLE[instance_key] as i32, 0);
        SHADOWBALL_CHARGE_HANDLE[instance_key] = 0;
    }
    // Clean up new sound handles
    if G_GRAB_BURN_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, G_GRAB_BURN_HANDLE[instance_key] as i32, 0);
        G_GRAB_BURN_HANDLE[instance_key] = 0;
    }
    if MEGASYMBOL_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, MEGASYMBOL_HANDLE[instance_key] as i32, 0);
        MEGASYMBOL_HANDLE[instance_key] = 0;
    }
    
    // Stop by name as backup
    SoundModule::stop_se(boma, Hash40::new("evolving"), 0);
    SoundModule::stop_se(boma, Hash40::new("evolve_ss"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_shadowball_charge"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_furafura"), 0);
    // Stop new sounds by name
    SoundModule::stop_se(boma, Hash40::new("g_grab_burn"), 0);
    SoundModule::stop_se(boma, Hash40::new("megasymbol"), 0);
    // Stop healing sounds on death/rebirth
    SoundModule::stop_se(boma, Hash40::new("g_potion"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_restore"), 0);
    
    // Reset all flags
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_FURAFURA_ACTIVE);
    // Reset new flags
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE);
    // Reset healing sound flags on death/rebirth
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);

    // Don't reset shiny flags during death/rebirth - let timers complete naturally
    // WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
    // WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);
    
    // Reset timers
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_FURAFURA_TIMER);
    // Reset new timers
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_GRAB_BURN_TIMER);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER);
    // Reset healing sound timers on death/rebirth
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
    
    // Clear healing detection tracker to prevent stale healing from triggering sounds after rebirth
    if instance_key < 256 {
        HEAL_DETECTED[instance_key] = (0.0, -200);
        // Reset status tracking to prevent false rebirth exit detection
        LAST_STATUS[instance_key] = -1;
        LAST_REBIRTH_EXIT_FRAME[instance_key] = -300;
    }
    
}

// Global state for all fighters playing as Purin/Gastly - using entry_id + color_id for full isolation
pub static FIGHTER_STATES: Lazy<RwLock<HashMap<u32, PlayerEvolutionState>>> = Lazy::new(|| RwLock::new(HashMap::new()));

// Helper function to create unique instance key (entry_id + costume color)
pub unsafe fn get_instance_key(boma: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as u32;
    
    // FIXED: Proper unique instance key for duos without collision
    let safe_entry_id = if entry_id < 8 { entry_id } else { 0 };
    let safe_color_id = color_id & 0xFF; // Limit to 8 bits
    
    // Use entry_id * 32 + color_id to ensure no collisions in duos
    // This gives each player a unique 32-slot range: P1(0-31), P2(32-63), etc.
    let result = (safe_entry_id * 32) + (safe_color_id % 32);
    
    // Ensure result fits in our array bounds (0-255)
    if result < 256 { result } else { 0 }
}

// Static tracking for grab effect cleanup - now using instance keys
static mut LAST_GRAB_STATUS: [i32; 256] = [-1; 256]; // Increased size for instance keys

// Static storage for tracking damage changes and heal detection - using instance keys for isolation
static mut DAMAGE_TRACKER: [(f32, i32); 256] = [(0.0, -200); 256]; // (last_damage, last_frame)
static mut HEAL_DETECTED: [(f32, i32); 256] = [(0.0, -200); 256]; // (heal_amount, frame_detected)
static mut LAST_DEATH_FRAME: [i32; 256] = [-300; 256]; // Track death/rebirth frames to prevent false healing detection
static mut LAST_STATUS: [i32; 256] = [-1; 256]; // Track status changes to detect rebirth exit
static mut LAST_REBIRTH_EXIT_FRAME: [i32; 256] = [-300; 256]; // Track when rebirth status was exited

#[skyline::hook(offset = 0x67A7B0)]
unsafe fn hit_tracking_hook(
    fighter_manager: u64,
    attacker_id: u32,
    defender_id: u32,
    move_type: u32,
    arg5: u32,
    move_type_again: u32,
    fighter: bool,
    arg8: u64
) -> u64 {
    let result = call_original!(fighter_manager, attacker_id, defender_id, move_type, arg5, move_type_again, fighter, arg8);

    let attacker_boma = smash::app::sv_battle_object::module_accessor(attacker_id);
    if !attacker_boma.is_null() && utility::get_kind(&mut *attacker_boma) == *FIGHTER_KIND_PURIN {
        let attacker_instance_key = get_instance_key(attacker_boma);

        let mut states_map_writer = FIGHTER_STATES.write();
        if let Some(player_state) = states_map_writer.get_mut(&attacker_instance_key) {
            if !player_state.is_evolving {
                player_state.hits_landed_this_stage += 1;
            }
        }
    }
    result
}

pub unsafe extern "C" fn gastly_global_fighter_frame(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }

    let fighter_kind_val: i32 = utility::get_kind(&mut *boma);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let instance_key = get_instance_key(boma);
    
    // Handle UI management and character switch detection
    if let Some(states_map_reader) = FIGHTER_STATES.try_read() {
        if let Some(player_state) = states_map_reader.get(&instance_key) {
            // We have a player state, which means this slot was Purin before
            drop(states_map_reader);  // Release read lock
            
            // Check if character has changed from Purin to something else
            if fighter_kind_val != *FIGHTER_KIND_PURIN {
                // Character switched away from Purin - clean up UI and remove player state
                if let Some(mut states_map_writer) = FIGHTER_STATES.try_write() {
                    if let Some(player_state) = states_map_writer.get_mut(&instance_key) {
                        // Call UI management one last time to clean up
                        handle_ui_management(boma, player_state, fighter);
                    }
                    // Remove the player state since we're no longer Purin
                    states_map_writer.remove(&instance_key);
                }
            } else {
                // Still Purin - normal UI management
                if let Some(mut states_map_writer) = FIGHTER_STATES.try_write() {
                    if let Some(player_state) = states_map_writer.get_mut(&instance_key) {
                        handle_ui_management(boma, player_state, fighter);
                    }
                }
            }
        }
    }
    
    // Only process dark moves when we're playing as Purin
    if fighter_kind_val == *FIGHTER_KIND_PURIN {
        
        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
        
        // Get current frame from player state
        let current_frame = {
            let states_map_reader = FIGHTER_STATES.read();
            states_map_reader.get(&instance_key)
                .map(|state| state.current_frame)
                .unwrap_or(0)
        };
        
        // Process dark moves with direct enemy access
        crate::gastly::darkfx::process_dark_move_effects(boma, fighter, current_frame);
    }
}

unsafe extern "C" fn init_gastly_aura(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }
    
    let fighter_kind_val: i32 = utility::get_kind(&mut *boma);
    if fighter_kind_val != *FIGHTER_KIND_PURIN { return; }
    
    // Initialize Gastly aura work module values
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_GASTLY_AURA_FRAME);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
}

//  Handle down special blink mesh visibility with proper mesh management
unsafe fn handle_down_special_blink(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) -> bool {
    let current_status = StatusModule::status_kind(boma);
    
    // Check if we're in down special (Rest) status
    if current_status == *FIGHTER_STATUS_KIND_SPECIAL_LW {
        let motion_frame = MotionModule::frame(boma);
        
        // Show blink mesh during frames 35-185
        if motion_frame >= 35.0 && motion_frame <= 185.0 {
            
            let blink_mesh = match player_state.stage {
                EvolutionStage::Gastly => *GASTLY_EYE_BLINK,
                EvolutionStage::Haunter => *HAUNTER_EYE_BLINK,
                EvolutionStage::Gengar => *GENGAR_EYE_BLINK,
            };
            
            // FIRST: Hide ALL animation-specific meshes that might interfere
            hide_all_animation_specific_meshes(boma);
            
            // SECOND: Ensure normal body parts are visible for current stage
            match player_state.stage {
                EvolutionStage::Gastly => { 
                    ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, true);
                }
                EvolutionStage::Haunter => {
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, true);
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, true);
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, true);
                }
                EvolutionStage::Gengar => {
                    ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, true);
                    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, true);
                    
                    // SPECIAL CASE: Hide Gastly body if in final smash form for Gengar/Haunter
                    if player_state.is_in_final_smash_form || 
                       WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) {
                        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
                    }
                }
            }
            
            // THIRD: Hide all other eye expressions for this stage
            match player_state.stage {
                EvolutionStage::Gastly => {
                    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                }
                EvolutionStage::Haunter => {
                    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                    // Also hide Gastly body for Haunter during final smash
                    if player_state.is_in_final_smash_form || 
                       WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) {
                        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
                    }
                }
                EvolutionStage::Gengar => {
                    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                }
            }
            
            // FOURTH: Show ONLY the blink mesh
            ModelModule::set_mesh_visibility(boma, blink_mesh, true);
            
            //        motion_frame, blink_mesh.hash);
            
            return true; // Indicate we're overriding eye expression for down special
        }
    }
    
    false // Not in down special blink range
}

// Handle win_3 motion blink sequences for all pokemon stages
unsafe fn handle_win_3_blink(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) -> bool {
    let current_status = StatusModule::status_kind(boma);
    let current_motion = MotionModule::motion_kind(boma);
    
    // Check if we're in win_3 or win_3_wait motions
    let is_win_3 = current_motion == smash::hash40("win_3");
    let is_win_3_wait = current_motion == smash::hash40("win_3_wait");
    
    if is_win_3 || is_win_3_wait {
        let motion_frame = MotionModule::frame(boma);
        let mut target_blink_mesh = None;
        
        if is_win_3 {
            // win_3: 1-113 blink, 114-155 open, 156 halfblink
            if motion_frame >= 1.0 && motion_frame <= 113.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_BLINK,
                    EvolutionStage::Haunter => *HAUNTER_EYE_BLINK,
                    EvolutionStage::Gengar => *GENGAR_EYE_BLINK,
                });
            } else if motion_frame >= 114.0 && motion_frame <= 155.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_N,
                    EvolutionStage::Haunter => *HAUNTER_EYE_N,
                    EvolutionStage::Gengar => *GENGAR_EYE_N,
                });
            } else if motion_frame >= 156.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_HALFBLINK1,
                    EvolutionStage::Haunter => *HAUNTER_EYE_HALFBLINK1,
                    EvolutionStage::Gengar => *GENGAR_EYE_HALFBLINK1,
                });
            }
        } else if is_win_3_wait {
            // win_3_wait: 1-52 halfblink, 53-207 blink, 208-251 open
            if motion_frame >= 1.0 && motion_frame <= 52.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_HALFBLINK1,
                    EvolutionStage::Haunter => *HAUNTER_EYE_HALFBLINK1,
                    EvolutionStage::Gengar => *GENGAR_EYE_HALFBLINK1,
                });
            } else if motion_frame >= 53.0 && motion_frame <= 207.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_BLINK,
                    EvolutionStage::Haunter => *HAUNTER_EYE_BLINK,
                    EvolutionStage::Gengar => *GENGAR_EYE_BLINK,
                });
            } else if motion_frame >= 208.0 && motion_frame <= 251.0 {
                target_blink_mesh = Some(match player_state.stage {
                    EvolutionStage::Gastly => *GASTLY_EYE_N,
                    EvolutionStage::Haunter => *HAUNTER_EYE_N,
                    EvolutionStage::Gengar => *GENGAR_EYE_N,
                });
            }
        }
        
        if let Some(blink_mesh) = target_blink_mesh {
            // FIRST: Hide ALL animation-specific meshes that might interfere
            hide_all_animation_specific_meshes(boma);
            
            // SECOND: Ensure normal body parts are visible for current stage
            match player_state.stage {
                EvolutionStage::Gastly => { 
                    ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, true);
                }
                EvolutionStage::Haunter => {
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, true);
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, true);
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, true);
                }
                EvolutionStage::Gengar => {
                    ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, true);
                    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, true);
                    
                    // SPECIAL CASE: Hide Gastly body if in final smash form for Gengar/Haunter
                    if player_state.is_in_final_smash_form || 
                       WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) {
                        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
                    }
                }
            }
            
            // THIRD: Hide all other eye expressions for this stage
            match player_state.stage {
                EvolutionStage::Gastly => {
                    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                }
                EvolutionStage::Haunter => {
                    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                    // Also hide Gastly body for Haunter during final smash
                    if player_state.is_in_final_smash_form || 
                       WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) {
                        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
                    }
                }
                EvolutionStage::Gengar => {
                    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() {
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
                    }
                }
            }
            
            // FOURTH: Show ONLY the target blink mesh
            ModelModule::set_mesh_visibility(boma, blink_mesh, true);
            
            return true; // Indicate we're overriding eye expression for win_3 motions
        }
    }
    
    false // Not in win_3 motion range
}

unsafe fn handle_evolution_readiness_icons(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState, fighter: &mut L2CFighterCommon) {
    let evolution_lockout_reset_duration: i32 = 5 * 60; // 5 seconds in frames

    let current_status_val = StatusModule::status_kind(boma);

    // Pre-checks: Suppress these icons under certain global states.
    let should_suppress_icons = player_state.is_evolving ||
                                player_state.everstone_effect_active ||
                                player_state.stage == EvolutionStage::Gengar ||
                                (player_state.linking_cord_evo_attempt_icon_is_pos_sensitive && player_state.linking_cord_evo_attempt_icon_timer > 0);

    if should_suppress_icons {
        if player_state.dmg_t_icon_display_timer > 0 || player_state.dmg_t_icon_is_locked_out ||
           player_state.dmg_d_icon_display_timer > 0 || player_state.dmg_d_icon_is_locked_out ||
           player_state.dmg_ss_icon_display_timer > 0 || player_state.dmg_ss_icon_is_locked_out ||
           player_state.dmg_se_icon_display_timer > 0 || player_state.dmg_se_icon_is_locked_out {
            player_state.reset_evo_readiness_icons();
        }
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);
        
        // Clean up charge bullet effects when suppressing icons
        EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_hold"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_start"), false, true);
        return;
    }

    let (required_dmg_received, required_hits) = match player_state.stage {
        EvolutionStage::Gastly => (
            GASTLY_EVO_DMG_RECEIVED_THRESHOLD + player_state.evo_attempt_delay_damage_taken_penalty,
            GASTLY_EVO_HITS_THRESHOLD + player_state.evo_attempt_delay_hits_penalty,
        ),
        EvolutionStage::Haunter => (
            HAUNTER_EVO_DMG_RECEIVED_THRESHOLD + player_state.evo_attempt_delay_damage_taken_penalty,
            HAUNTER_EVO_HITS_THRESHOLD + player_state.evo_attempt_delay_hits_penalty,
        ),
        EvolutionStage::Gengar => return,
    };

    // ENHANCED VALIDATION: Prevent hit-count readiness desync and penalty accumulation
    // Base thresholds without penalties for validation
    let base_hits_threshold = match player_state.stage {
        EvolutionStage::Gastly => GASTLY_EVO_HITS_THRESHOLD,
        EvolutionStage::Haunter => HAUNTER_EVO_HITS_THRESHOLD,
        EvolutionStage::Gengar => return,
    };
    
    // If penalties are unreasonably high, reset them (prevents accumulation bugs)
    if player_state.evo_attempt_delay_hits_penalty > base_hits_threshold {
        player_state.evo_attempt_delay_hits_penalty = 0;
    }
    if player_state.evo_attempt_delay_damage_taken_penalty > 50.0 { // 50% is max reasonable penalty
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
    }
    
    // Use a conservative maximum for hit validation (base + 10, not based on penalties)
    let max_reasonable_hits = base_hits_threshold + 10;
    if player_state.hits_landed_this_stage > max_reasonable_hits {
        // Reset both hit count and penalties if something is clearly wrong
        player_state.hits_landed_this_stage = 0;
        player_state.damage_received_this_stage = 0.0;
        player_state.evo_attempt_delay_hits_penalty = 0;
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
    }

    let damage_condition_met = player_state.damage_received_this_stage >= required_dmg_received;
    let hits_condition_met = player_state.hits_landed_this_stage >= required_hits;
    let both_conditions_met = damage_condition_met && hits_condition_met;

    // Debug evolution progress every 60 frames
    if player_state.current_frame % 60 == 0 && (damage_condition_met || hits_condition_met) {
    }

    // --- Logic for resetting "lockout" flags ---
    //  Only reset lockout after evolution OR significant drop in progress (training reset)
    let significant_progress_drop = 
        (player_state.damage_received_this_stage < required_dmg_received * 0.5) ||
        (player_state.hits_landed_this_stage < required_hits / 2);
    
    if player_state.last_evolution_confirmation_frame != -1 &&
       (player_state.current_frame - player_state.last_evolution_confirmation_frame >= evolution_lockout_reset_duration) {
        player_state.reset_evo_readiness_icons();
        player_state.last_evolution_confirmation_frame = -1;
        
        //  Reset flash flag after evolution timeout to allow new flash
        let instance_key = get_instance_key(boma) as usize;
        if (instance_key as usize) < 256 {
            READINESS_FLASH_OCCURRED[instance_key] = false;
        }
        
    }

    //  Only unlock lockouts when conditions are ACTUALLY no longer met
    // This prevents damage icon from persistently flashing when damage stays high
    if !damage_condition_met && player_state.dmg_t_icon_is_locked_out {
        player_state.dmg_t_icon_is_locked_out = false;
    }
    if !hits_condition_met && player_state.dmg_d_icon_is_locked_out {
        player_state.dmg_d_icon_is_locked_out = false;
    }
    if !damage_condition_met || !hits_condition_met {
        if player_state.dmg_ss_icon_is_locked_out {
            player_state.dmg_ss_icon_is_locked_out = false;
        }
        if player_state.dmg_se_icon_is_locked_out {
            player_state.dmg_se_icon_is_locked_out = false;
        }
    }
    
    // Reset flash flag only on significant progress drop (training reset)
    if significant_progress_drop {
        let instance_key = get_instance_key(boma) as usize;
        if (instance_key as usize) < 256 {
            READINESS_FLASH_OCCURRED[instance_key] = false;
        }
    }

    // --- Icon Trigger and Precedence Logic ---
    //  Track if flash already occurred for readiness icons to prevent spam
    static mut READINESS_FLASH_OCCURRED: [bool; 256] = [false; 256];
    
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    let instance_key = get_instance_key(boma) as usize;

    let mut ss_triggered_this_frame = false;
    let mut t_triggered_this_frame = false;

    // 1. Try to trigger Both Conditions Met (SS -> SE sequence) FIRST - both Gastly and Haunter stages
    if both_conditions_met && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) {
        if player_state.dmg_ss_icon_display_timer == 0 && !player_state.dmg_ss_icon_is_locked_out &&
           player_state.dmg_se_icon_display_timer == 0 && !player_state.dmg_se_icon_is_locked_out {
            
            player_state.dmg_ss_icon_display_timer = READINESS_ICON_DURATION;
            ss_triggered_this_frame = true;
            // Only flash during normal gameplay, not standby/entry - AND only if not already flashed
            if current_status_val != *FIGHTER_STATUS_KIND_STANDBY && 
               current_status_val != *FIGHTER_STATUS_KIND_ENTRY &&
               instance_key < 256 && !READINESS_FLASH_OCCURRED[instance_key] {
                FighterUtil::flash_eye_info(boma);
                READINESS_FLASH_OCCURRED[instance_key] = true;
            }

            // Add sys_counter_flash effect when SS icon triggers
            macros::EFFECT_FLW_POS(
                fighter,
                Hash40::new("sys_counter_flash"),
                Hash40::new("body"),
                0, 0, 0, 0, 90, 0, 0.3,
                true
            );
            macros::LAST_EFFECT_SET_RATE(fighter, 0.4);
            macros::LAST_EFFECT_SET_ALPHA(fighter, 0.5);

            player_state.dmg_t_icon_display_timer = 0;
            player_state.dmg_t_icon_is_locked_out = true;
            ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);

            player_state.dmg_d_icon_display_timer = 0;
            player_state.dmg_d_icon_is_locked_out = true;
            ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        }
    }

    let ss_or_se_is_displaying_or_just_triggered = ss_triggered_this_frame ||
                                                 player_state.dmg_ss_icon_display_timer > 0 ||
                                                 player_state.dmg_se_icon_display_timer > 0;

    // 2. Try to trigger Only Damage Condition Met (T icon - both Gastly and Haunter stages)
    if !ss_or_se_is_displaying_or_just_triggered { 
        if damage_condition_met && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) &&
           player_state.dmg_t_icon_display_timer == 0 && !player_state.dmg_t_icon_is_locked_out {
            player_state.dmg_t_icon_display_timer = READINESS_ICON_DURATION;
            t_triggered_this_frame = true;
            // Only flash during normal gameplay, not standby/entry - AND only if not already flashed
            if current_status_val != *FIGHTER_STATUS_KIND_STANDBY && 
               current_status_val != *FIGHTER_STATUS_KIND_ENTRY &&
               instance_key < 256 && !READINESS_FLASH_OCCURRED[instance_key] {
                FighterUtil::flash_eye_info(boma);
                READINESS_FLASH_OCCURRED[instance_key] = true;
            }
        }
    }

    // 3. Try to trigger Only Hits Condition Met (D icon - both Gastly and Haunter stages)
    if !ss_or_se_is_displaying_or_just_triggered && !t_triggered_this_frame {
        if hits_condition_met && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) &&
           player_state.dmg_d_icon_display_timer == 0 && !player_state.dmg_d_icon_is_locked_out {
            player_state.dmg_d_icon_display_timer = READINESS_ICON_DURATION;
            // Only flash during normal gameplay, not standby/entry - AND only if not already flashed
            if current_status_val != *FIGHTER_STATUS_KIND_STANDBY && 
               current_status_val != *FIGHTER_STATUS_KIND_ENTRY &&
               instance_key < 256 && !READINESS_FLASH_OCCURRED[instance_key] {
                FighterUtil::flash_eye_info(boma);
                READINESS_FLASH_OCCURRED[instance_key] = true;
            }
        }
    }

    // --- Timer Decrement, Visibility Update, and Lockout/Next Icon Activation ---

    // STG1_DMG_T (show for both Gastly and Haunter stages)
    if player_state.dmg_t_icon_display_timer > 0 && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) {
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, true);
        player_state.dmg_t_icon_display_timer -= 1;
        if player_state.dmg_t_icon_display_timer == 0 {
            player_state.dmg_t_icon_is_locked_out = true;
        }
    } else {
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
    }

    // STG1_DMG_D (show for both Gastly and Haunter stages)
    if player_state.dmg_d_icon_display_timer > 0 && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) {
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, true);
        player_state.dmg_d_icon_display_timer -= 1;
        if player_state.dmg_d_icon_display_timer == 0 {
            player_state.dmg_d_icon_is_locked_out = true;
        }
    } else {
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
    }

    // STG2_DMG_SS with Charge Bullet Hold effect (show for both Gastly and Haunter stages)
    if player_state.dmg_ss_icon_display_timer > 0 && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) {
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, true);

        //  DON'T play evolve_ss sound here - let the persistent sound system handle it
        // The persistent sound system will detect dmg_ss_icon_display_timer > 0 and play the sound
        
        // Spawn bayonetta_chargebullet_hold effect during SS icon visibility
        static mut LAST_SS_CHARGEBULLET_FRAME: [i32; 256] = [-30; 256];
        if (instance_key as usize) < 256 && (player_state.current_frame - LAST_SS_CHARGEBULLET_FRAME[instance_key] >= 30) {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("bayonetta_chargebullet_hold"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                1.0,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                EffectModule::set_rgb(boma, handle, 1.0, 1.0, 1.0);
                EffectModule::set_alpha(boma, handle, 0.2);
                LAST_SS_CHARGEBULLET_FRAME[instance_key] = player_state.current_frame;
            }
        }
        
        player_state.dmg_ss_icon_display_timer -= 1;
        if player_state.dmg_ss_icon_display_timer == 0 {
            player_state.dmg_ss_icon_is_locked_out = true;
            
            // Clean up charge bullet hold effect when SS icon ends
            EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_hold"), false, true);
            
            if both_conditions_met &&
               player_state.dmg_se_icon_display_timer == 0 && !player_state.dmg_se_icon_is_locked_out {
                player_state.dmg_se_icon_display_timer = READINESS_ICON_DURATION;
                player_state.dmg_se_icon_is_locked_out = false;
                // Only flash during normal gameplay, not standby/entry - AND only if not already flashed
                if current_status_val != *FIGHTER_STATUS_KIND_STANDBY && 
                   current_status_val != *FIGHTER_STATUS_KIND_ENTRY &&
                   instance_key < 256 && !READINESS_FLASH_OCCURRED[instance_key] {
                    FighterUtil::flash_eye_info(boma);
                    READINESS_FLASH_OCCURRED[instance_key] = true;
                }
            }
        }
    } else {
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        // Clean up charge bullet hold effect when SS icon not visible or wrong stage
        if player_state.dmg_se_icon_display_timer == 0 || 
           (player_state.stage != EvolutionStage::Gastly && player_state.stage != EvolutionStage::Haunter) {
            EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_hold"), false, true);
        }
    }

    // Static variables for SE charge bullet effects (moved to function scope)
    static mut SE_CHARGEBULLET_START_SPAWNED: [bool; 256] = [false; 256];
    static mut LAST_SE_CHARGEBULLET_FRAME: [i32; 256] = [-30; 256];
    
    // STG2_DMG_SE with both Charge Bullet effects (show for both Gastly and Haunter stages)
    if player_state.dmg_se_icon_display_timer > 0 && (player_state.stage == EvolutionStage::Gastly || player_state.stage == EvolutionStage::Haunter) {
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, true);
        
        let instance_key = get_instance_key(boma) as usize;
        
        if (instance_key as usize) < 256 {
            // Spawn start effect only once when SE icon first becomes visible
            if player_state.dmg_se_icon_display_timer == READINESS_ICON_DURATION && !SE_CHARGEBULLET_START_SPAWNED[instance_key] {
                //  Play evolve_se sound when SE icon appears (NOT through persistent system)
                crate::gastly::persist_sfx::play_evolve_se_sound(boma);
                
                let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
                
                let start_handle = EffectModule::req_follow(
                    boma,
                    Hash40::new("bayonetta_chargebullet_start"),
                    Hash40::new("body"),
                    &position_offset,
                    &rotation_vector,
                    1.0,
                    true, 0x40000, 0, -1, 0, 0, false, false
                ) as u32;
                
                if start_handle != u64::MAX as u32 && start_handle != 0 {
                    SE_CHARGEBULLET_START_SPAWNED[instance_key] = true;
                }
            }
            
            // Continue spawning hold effect during SE icon visibility (same as SS)
            if player_state.current_frame - LAST_SE_CHARGEBULLET_FRAME[instance_key] >= 30 {
                let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
                
                let hold_handle = EffectModule::req_follow(
                    boma,
                    Hash40::new("bayonetta_chargebullet_hold"),
                    Hash40::new("body"),
                    &position_offset,
                    &rotation_vector,
                    1.0,
                    true, 0x40000, 0, -1, 0, 0, false, false
                ) as u32;
                
                if hold_handle != u64::MAX as u32 && hold_handle != 0 {
                    EffectModule::set_rgb(boma, hold_handle, 1.0, 1.0, 1.0);
                    EffectModule::set_alpha(boma, hold_handle, 0.2);
                    LAST_SE_CHARGEBULLET_FRAME[instance_key] = player_state.current_frame;
                }
            }
        }
        
        player_state.dmg_se_icon_display_timer -= 1;
        if player_state.dmg_se_icon_display_timer == 0 {
            player_state.dmg_se_icon_is_locked_out = true;
            
            // Clean up charge bullet effects when SE icon ends
            EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_hold"), false, true);
            EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_start"), false, true);
            
            // Reset start effect flag
            if (instance_key as usize) < 256 {
                SE_CHARGEBULLET_START_SPAWNED[instance_key] = false;
            }
            
            //  Evolution will be triggered in the next call to handle_evolution_process
            // The persistent sound system will detect is_evolving and start the evolving sound
        }
    } else {
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);
        
        // Clean up charge bullet effects when SE icon not visible or wrong stage
        let instance_key = get_instance_key(boma) as usize;
        if (instance_key as usize) < 256 {
            // Clean up if SS also not active, or if we're not in valid stage
            if player_state.dmg_ss_icon_display_timer == 0 || 
               (player_state.stage != EvolutionStage::Gastly && player_state.stage != EvolutionStage::Haunter) {
                EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_hold"), false, true);
            }
            // Always clean up start effect when SE not visible or wrong stage
            EffectModule::kill_kind(boma, Hash40::new("bayonetta_chargebullet_start"), false, true);
            SE_CHARGEBULLET_START_SPAWNED[instance_key] = false;
        }
    }

    // Track previous state for comparison for condition sounds
    static mut PREV_DAMAGE_MET: [bool; 256] = [false; 256];
    static mut PREV_HITS_MET: [bool; 256] = [false; 256];
    static mut FIRST_CONDITION_PLAYED: [bool; 256] = [false; 256];

    let instance_key = get_instance_key(boma) as usize;
    if (instance_key as usize) < 256 {
        let prev_damage = PREV_DAMAGE_MET[instance_key];
        let prev_hits = PREV_HITS_MET[instance_key];
        let prev_any_condition = prev_damage || prev_hits;
        let prev_both_conditions = prev_damage && prev_hits;
        
        // First condition met (transition from 0 to 1 condition)
        if !prev_any_condition && (damage_condition_met || hits_condition_met) && !FIRST_CONDITION_PLAYED[instance_key] {
            crate::gastly::persist_sfx::play_condition_sound(boma, 1);
            FIRST_CONDITION_PLAYED[instance_key] = true;
        }
        
        // Second condition met (transition from 1 to 2 conditions)
        if !prev_both_conditions && both_conditions_met && FIRST_CONDITION_PLAYED[instance_key] {
            crate::gastly::persist_sfx::play_condition_sound(boma, 2);
        }
        
        // Reset flags when conditions are no longer met (evolution occurred or reset)
        if !damage_condition_met && !hits_condition_met {
            FIRST_CONDITION_PLAYED[instance_key] = false;
        }
        
        
        // Update tracking
        PREV_DAMAGE_MET[instance_key] = damage_condition_met;
        PREV_HITS_MET[instance_key] = hits_condition_met;
    }
}

// Shiny detection
unsafe fn detect_shiny_slot(boma: *mut BattleObjectModuleAccessor) -> bool {
    crate::is_shiny_gastly_costume(boma)
}

unsafe fn spawn_shiny_effect(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    current_frame: i32
) {
    // Spawn shiny sparkle sound (102 frames) - managed as looping sound
    WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);

    let sparkle_handle = SoundModule::play_se(
        boma,
        Hash40::new("shiny_sparkle"),
        true, // Loop=true since it's a looping file
        false, false, false,
        smash::app::enSEType(0)
    );
    SoundModule::set_se_vol(boma, sparkle_handle as i32, 1.5, 0);
    
    // Spawn visual effect (90 frames)
    let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
    let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
    
    let effect_handle = EffectModule::req_follow(
        boma,
        Hash40::new("rosetta_tico_happy_light"),
        Hash40::new("body"),
        &position_offset,
        &rotation_vector,
        1.0,
        true, 0x40000, 0, -1, 0, 0, false, false
    ) as u32;
    
    if effect_handle != u64::MAX as u32 && effect_handle != 0 {
        EffectModule::set_rate(boma, effect_handle, 0.7);
        WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);
        WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        
    }
}

// Shiny effect handling
unsafe fn handle_shiny_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    current_status: i32,
    current_frame: i32
) {
    if !player_state.is_shiny { return; }
    
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    
    // Check for Gastly stage triggers (ENTRY only)
    if player_state.stage == EvolutionStage::Gastly {
        let should_trigger = current_status == *FIGHTER_STATUS_KIND_ENTRY;
        
        static mut LAST_GASTLY_STATUS: [i32; 256] = [-1; 256];
        let instance_key = get_instance_key(boma) as usize;
        let status_just_changed = instance_key < 256 && LAST_GASTLY_STATUS[instance_key] != current_status;
        
        if should_trigger && status_just_changed {
            spawn_shiny_effect(boma, player_state, current_frame);
        }
        
        if (instance_key as usize) < 256 {
            LAST_GASTLY_STATUS[instance_key] = current_status;
        }
    }
    
    // Check for post-evolution triggers (75 frames after evolution completion)
    if player_state.shiny_effect_pending {
        if player_state.shiny_effect_delay_timer > 0 {
            player_state.shiny_effect_delay_timer -= 1;
        } else if player_state.shiny_effect_delay_timer == 0 {
            // For rebirth delays, check if we should skip due to pokecenter
            let should_skip_for_pokecenter = player_state.stage == EvolutionStage::Gastly && 
                                            current_status == *FIGHTER_STATUS_KIND_REBIRTH &&
                                            entry_id < 8;
            
            // Always trigger shiny effect on rebirth, regardless of pokecenter
            spawn_shiny_effect(boma, player_state, current_frame);
            
            player_state.shiny_effect_pending = false;
            player_state.shiny_effect_delay_timer = -1;
        }
    }
    
    // Handle shiny effect duration (90 frames)
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE) {
        let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        let new_timer = timer + 1.0;
        WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        
        if new_timer >= 90.0 {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);
            EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_happy_light"), false, true);
        }
    }
}

pub unsafe extern "C" fn gastly_fighter_frame_callback(fighter: &mut L2CFighterCommon) {
    static mut POKECENTER_HIGHEST_DAMAGE: [f32; 256] = [0.0; 256];
    static mut POKECENTER_PLAYED: [bool; 256] = [false; 256];
    static mut POKECENTER_LAST_STATUS: [i32; 256] = [-1; 256];

    let boma = fighter.module_accessor;
    if boma.is_null() { return; }

    let fighter_kind_val: i32 = utility::get_kind(&mut *boma);
    
    if fighter_kind_val != *FIGHTER_KIND_PURIN { return; }
    

    let my_entry_id_i32 = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let my_entry_id_u32 = my_entry_id_i32 as u32;
    let instance_key = get_instance_key(boma);
    
    // FIRST ACCESS: Reset marked costumes on first HashMap access after training reset

    // TRAINING RESET: Detect first callback after reset for marked slots
    static mut CALLBACK_COUNT: [i32; 256] = [0; 256];
    static mut LAST_RESET_CALLBACK: [i32; 256] = [-1; 256];
    let instance_idx = instance_key as usize;
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    let is_marked_slot = if color_id < 256 {
        unsafe { crate::MARKED_COLORS[color_id] }
    } else {
        false
    };

    if instance_idx < 256 && is_marked_slot {
        CALLBACK_COUNT[instance_idx] += 1;
        
        // If callback count reset to 1, it's likely a training reset
        if CALLBACK_COUNT[instance_idx] == 1 && LAST_RESET_CALLBACK[instance_idx] > 100 {
            let mut states_map_writer = FIGHTER_STATES.write();
            if let Some(player_state) = states_map_writer.get_mut(&instance_key) {
                if player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
                    
                    player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.is_evolving = false;
                    player_state.evolution_timer = 0;
                    player_state.is_in_final_smash_form = false;
                    player_state.mega_gengar_form_active = false;
                    player_state.giga_gengar_form_active = false;
                    
                    // Force visual update immediately
                    crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, crate::gastly::player_state::EvolutionStage::Gastly);
                    crate::gastly::visuals::set_active_eye_mesh(boma, player_state, None);
                    
                }
            }
        }
        
        // Store the callback count for reset detection
        if CALLBACK_COUNT[instance_idx] % 60 == 0 {
            LAST_RESET_CALLBACK[instance_idx] = CALLBACK_COUNT[instance_idx];
        }
    }

    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;

    if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
        static mut FIRST_ACCESS_THIS_BOOT: [bool; 256] = [false; 256];
        static mut LAST_DAMAGE_SEEN: [f32; 256] = [-1.0; 256];
        
        let instance_idx = instance_key as usize;
        if instance_idx < 256 {
            let current_damage = DamageModule::damage(boma, 0);
            
            // Detect training reset: damage went from high to 0
            // BUT NOT during results screen
            let current_status = StatusModule::status_kind(boma);
            let is_results_screen = current_status == *FIGHTER_STATUS_KIND_WIN ||
                                current_status == *FIGHTER_STATUS_KIND_LOSE ||
                                current_status == 0x107 ||
                                (current_status >= 0x190 && current_status <= 0x1DC);

            let damage_reset_detected = LAST_DAMAGE_SEEN[instance_idx] > 20.0 && 
                                    current_damage <= 0.1 && 
                                    !is_results_screen;
            
            // Reset first access flag when training reset detected (but not during results)
            if damage_reset_detected {
                FIRST_ACCESS_THIS_BOOT[instance_idx] = false;
            } else if is_results_screen {
            }
            
            // On very first access OR first access after training reset, force Gastly
            if !FIRST_ACCESS_THIS_BOOT[instance_idx] {
                
                // Force create new Gastly state and handle evolution cancellation
                let mut states_map_writer = FIGHTER_STATES.write();
                
                // Check if we had an existing state that was evolving
                let was_evolving = states_map_writer.get(&instance_key)
                    .map(|state| state.is_evolving)
                    .unwrap_or(false);
                
                // Cancel evolution sounds if evolving
                if was_evolving {
                    SoundModule::stop_se(boma, Hash40::new("evolving"), 0);
                    SoundModule::stop_se(boma, Hash40::new("evolve_ss"), 0);
                }
                
                states_map_writer.remove(&instance_key); // Remove any existing state
                let new_state = states_map_writer.entry(instance_key).or_insert_with(PlayerEvolutionState::new);
                // new_state is automatically Gastly stage with is_evolving = false
                
                // Reset damage tracking to prevent stale healing detection from previous session
                crate::gastly::reset_damage_tracker_for_entry(boma);
                crate::gastly::reset_heal_tracker_for_entry(boma);
                
                // Force visual update
                crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, crate::gastly::player_state::EvolutionStage::Gastly);
                crate::gastly::visuals::set_active_eye_mesh(boma, new_state, None);
                
                FIRST_ACCESS_THIS_BOOT[instance_idx] = true;
            }
            
            // Update damage tracking
            if current_damage > 0.1 {
                LAST_DAMAGE_SEEN[instance_idx] = current_damage;
            }
        }
    }

    // Clean up lingering walk/run sounds when not in walk/run status
    let current_status = StatusModule::status_kind(boma);
    let is_walk_or_run_status = current_status == *FIGHTER_STATUS_KIND_WALK || 
                               current_status == *FIGHTER_STATUS_KIND_RUN;
    
    if !is_walk_or_run_status {
        // Stop all walk/run sounds when not in walk/run status
        SoundModule::stop_se(boma, Hash40::new("g_walkslow"), 0);
        SoundModule::stop_se(boma, Hash40::new("g_walkmiddle"), 0);
        SoundModule::stop_se(boma, Hash40::new("g_walkfast"), 0);
        SoundModule::stop_se(boma, Hash40::new("g_run"), 0);
        
        // Also stop vanilla step sounds for Gengar as backup
        SoundModule::stop_se(boma, Hash40::new("se_purin_step_right_s"), 0);
        SoundModule::stop_se(boma, Hash40::new("se_purin_step_left_s"), 0);
        SoundModule::stop_se(boma, Hash40::new("se_purin_step_right_m"), 0);
        SoundModule::stop_se(boma, Hash40::new("se_purin_step_left_m"), 0);
    }

    // First-frame detection for marked costumes
    static mut FIRST_FRAME_PROCESSED: [bool; 256] = [false; 256];
    let instance_idx_first = instance_key as usize;
    let current_status_val = StatusModule::status_kind(boma);

    if instance_idx_first < 256 && !FIRST_FRAME_PROCESSED[instance_idx_first] {
        FIRST_FRAME_PROCESSED[instance_idx_first] = true;
        
        let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
            // Force complete state reset for marked costumes
            let mut states_map_reset = FIGHTER_STATES.write();
            if let Some(state) = states_map_reset.get_mut(&instance_key) {
                state.full_reset_on_respawn(boma);
                state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
            }
        }
    }

    // Reset first frame flag on death/standby
    if current_status_val == *FIGHTER_STATUS_KIND_DEAD || 
    current_status_val == *FIGHTER_STATUS_KIND_STANDBY {
        if instance_idx_first < 256 {
            FIRST_FRAME_PROCESSED[instance_idx_first] = false;
        }
    }

    // ===== POKECENTER LOGIC (for Purin only) =====
    let current_status_val: i32 = StatusModule::status_kind(boma);

    let current_damage = DamageModule::damage(boma, 0);
    let instance_idx_pokecenter = instance_key as usize; // Convert to usize for array indexing

    if instance_idx_pokecenter < 256 {
        // RESET DETECTION - Only reset on actual training mode resets, NOT on death
        let should_reset_pokecenter = 
            // Match start statuses (actual resets)
            (current_status_val == *FIGHTER_STATUS_KIND_STANDBY) ||
            (current_status_val == *FIGHTER_STATUS_KIND_ENTRY) ||
            // Additional reset detection for training re-entry (but not during rebirth)
            (current_damage <= 0.1 && current_status_val != *FIGHTER_STATUS_KIND_REBIRTH);
        
        if should_reset_pokecenter {
            if POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] > 50.0 { // Only log if we're actually resetting something
            }
            
            POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] = 0.0;
            POKECENTER_PLAYED[instance_idx_pokecenter] = false;  // ← KEY FIX: Reset the played flag
            POKECENTER_LAST_STATUS[instance_idx_pokecenter] = -1;
            
            // Get current frame from player state
            let current_frame = {
                let states_map = FIGHTER_STATES.read();
                if let Some(state) = states_map.get(&instance_key) {
                    state.current_frame
                } else {
                    0 // Fallback
                }
            };
            
            // Call the existing reset function for damage/heal tracking  
            let instance_key_for_reset = get_instance_key(boma) as usize;
            reset_all_match_tracking_by_instance_key(instance_key_for_reset, current_frame);
        }
        
        // Track highest damage
        if current_damage > POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] {
            POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] = current_damage;
            if current_damage >= 90.0 {
            }
        }
        
        // ADDITIONAL RESET: Only reset PLAYED flag after sound plays and we're back to low damage
        if current_damage <= 5.0 && POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] >= 100.0 && POKECENTER_PLAYED[instance_idx_pokecenter] {
            // Wait a few frames after damage goes to 0, then reset ONLY the played flag for next session
            static mut RESET_DELAY_TIMER: [i32; 256] = [0; 256];
            RESET_DELAY_TIMER[instance_idx_pokecenter] += 1;
            
            if RESET_DELAY_TIMER[instance_idx_pokecenter] >= 300 { // 5 seconds after damage goes to 0
                    POKECENTER_PLAYED[instance_idx_pokecenter] = false; // Only reset the played flag, keep highest damage
                RESET_DELAY_TIMER[instance_idx_pokecenter] = 0;
            }
        } else {
            static mut RESET_DELAY_TIMER: [i32; 256] = [0; 256];
            RESET_DELAY_TIMER[instance_idx_pokecenter] = 0; // Reset timer if damage is not at 0
        }
        
        // Check for rebirth
        if current_status_val == *FIGHTER_STATUS_KIND_REBIRTH && POKECENTER_LAST_STATUS[instance_idx_pokecenter] != current_status_val {
            
            if POKECENTER_HIGHEST_DAMAGE[instance_idx_pokecenter] >= 100.0 && !POKECENTER_PLAYED[instance_idx_pokecenter] {
                let sfx_handle = SoundModule::play_se(
                    boma,
                    Hash40::new("g_pokecenter"),
                    false, false, false, false,
                    smash::app::enSEType(0)
                );
                SoundModule::set_se_vol(boma, sfx_handle as i32, 3.0, 0);
                POKECENTER_PLAYED[instance_idx_pokecenter] = true;
                
            } else {
            }
        }
        
        POKECENTER_LAST_STATUS[instance_idx_pokecenter] = current_status_val;
    }

    let mut states_map_writer = FIGHTER_STATES.write();
    let is_new_state = !states_map_writer.contains_key(&instance_key);

    // Initialize shiny detection for new players
    if is_new_state {
        let player_state = states_map_writer.entry(instance_key).or_insert_with(PlayerEvolutionState::new);
        player_state.is_shiny = detect_shiny_slot(boma);
        if player_state.is_shiny {
            }
    } else {
        // For existing players, ensure shiny detection is set
        if let Some(player_state) = states_map_writer.get_mut(&instance_key) {
            if !player_state.is_shiny {
                player_state.is_shiny = detect_shiny_slot(boma);
            }
        }
    }

    let player_state = states_map_writer.entry(instance_key).or_insert_with(PlayerEvolutionState::new);

    // Check for resets FIRST, before any other logic  
    let was_evolving_before_reset = player_state.is_evolving;
    reset_evolution_progress_on_match_start(boma, player_state);
    let was_reset_triggered = was_evolving_before_reset && !player_state.is_evolving;

    // Debug evolution state tracking
    if was_evolving_before_reset {
    }

    // Force cancel evolution if reset occurred while evolving
    if was_reset_triggered {
        player_state.cancel_evolution(fighter);
    }

    // Check for new training session before any other logic
    if detect_new_training_session_for_marked_costumes(boma, player_state, my_entry_id_u32) {

    // AGGRESSIVE: Force Gastly reset for marked costumes in early frames
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
        static mut EARLY_RESET_APPLIED: [bool; 256] = [false; 256];
        let instance_idx_early = instance_key as usize;
        
        if instance_idx_early < 256 {
            let current_damage = DamageModule::damage(boma, 0);
            
            // Force reset on early frames with 0 damage if not already Gastly
            if player_state.current_frame < 60 && 
            current_damage <= 0.1 && 
            !EARLY_RESET_APPLIED[instance_idx_early] &&
            player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
                
                player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
                player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
                player_state.is_evolving = false;
                player_state.evolution_timer = 0;
                player_state.is_in_final_smash_form = false;
                player_state.mega_gengar_form_active = false;
                player_state.giga_gengar_form_active = false;
                
                EARLY_RESET_APPLIED[instance_idx_early] = true;
                
                
                // Force visual update
                crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, crate::gastly::player_state::EvolutionStage::Gastly);
                crate::gastly::visuals::set_active_eye_mesh(boma, player_state, None);
            }
            
            // Reset the flag once we're past early frames
            if player_state.current_frame > 120 {
                EARLY_RESET_APPLIED[instance_idx_early] = false;
            }
        }
    }
}

    
    // ENHANCED: Handle Gastly aura for special situations (rebirth + results screen)
    if player_state.stage == crate::gastly::player_state::EvolutionStage::Gastly {
        // NEW APPROACH: Use broader detection methods
        let needs_forced_aura = current_status_val == *FIGHTER_STATUS_KIND_REBIRTH || // Rebirth platform
                               current_status_val == *FIGHTER_STATUS_KIND_WIN ||     // Win pose (0x1DA)
                               current_status_val == *FIGHTER_STATUS_KIND_LOSE ||    // Lose pose (0x1DB) 
                               current_status_val == *FIGHTER_STATUS_KIND_ENTRY ||   // Entry animation (0x1D9)
                               (current_status_val >= 0x190 && current_status_val <= 0x1A0) || // Results range
                               (current_status_val >= 0x1D9 && current_status_val <= 0x1DC); // Extended results range
        
        // ALTERNATIVE: Check motion hashes for results screen detection
        let current_motion = MotionModule::motion_kind(boma);
        let is_results_motion = current_motion == 0x7fb997a80 ||  // Known results motion 1
                       current_motion == smash::hash40("result") ||
                       current_motion == smash::hash40("result_normal") ||
                       current_motion == smash::hash40("result_pose");
        
        let is_problematic_result_motion = current_motion == 0x42af5a458; // No contest motion
        let needs_aura = needs_forced_aura || (is_results_motion && !is_problematic_result_motion);
        
        if needs_aura {
            static mut SPECIAL_AURA_SPAWNED: [bool; 256] = [false; 256];
            static mut LAST_SPECIAL_STATUS: [i32; 256] = [-1; 256];
            static mut LAST_SPECIAL_MOTION: [u64; 256] = [0; 256];
            static mut SPECIAL_AURA_HANDLE: [u32; 256] = [0; 256];
            let instance_idx_aura = instance_key as usize;
            
            if instance_idx_aura < 256 {
                // Reset spawn flag when status OR motion changes
                if LAST_SPECIAL_STATUS[instance_idx_aura] != current_status_val || 
                   LAST_SPECIAL_MOTION[instance_idx_aura] != current_motion {
                    SPECIAL_AURA_SPAWNED[instance_idx_aura] = false;
                    LAST_SPECIAL_STATUS[instance_idx_aura] = current_status_val;
                    LAST_SPECIAL_MOTION[instance_idx_aura] = current_motion;
                }
                
                // Always try to maintain aura during these states
                if !SPECIAL_AURA_SPAWNED[instance_idx_aura] || 
                   (SPECIAL_AURA_HANDLE[instance_idx_aura] != 0 && !EffectModule::is_exist_effect(boma, SPECIAL_AURA_HANDLE[instance_idx_aura])) {
                    
                    
                    // Clean up any existing aura first (both normal and special)
                    let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
                    if stored_handle != 0 && EffectModule::is_exist_effect(boma, stored_handle) {
                        EffectModule::kill(boma, stored_handle, false, true);
                        WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID);
                    }
                    
                    // Kill our old special handle too
                    if SPECIAL_AURA_HANDLE[instance_idx_aura] != 0 && EffectModule::is_exist_effect(boma, SPECIAL_AURA_HANDLE[instance_idx_aura]) {
                        EffectModule::kill(boma, SPECIAL_AURA_HANDLE[instance_idx_aura], false, true);
                        SPECIAL_AURA_HANDLE[instance_idx_aura] = 0;
                    }
                    
                    // Spawn new aura with extra persistence settings
                    let handle = crate::gastly::effects::spawn_gastly_aura_direct(boma);
                    if handle != 0 {
                        SPECIAL_AURA_HANDLE[instance_idx_aura] = handle;
                        SPECIAL_AURA_SPAWNED[instance_idx_aura] = true;
                        
                        // Set visibility and persistence flags
                        EffectModule::set_visible(boma, handle, true);
                        
                        // Set a flag to prevent normal aura system from interfering
                        WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
                        
                    }
                }
                
                // Continuously ensure our special aura stays visible during these states
                if SPECIAL_AURA_HANDLE[instance_idx_aura] != 0 {
                    if EffectModule::is_exist_effect(boma, SPECIAL_AURA_HANDLE[instance_idx_aura]) {
                        EffectModule::set_visible(boma, SPECIAL_AURA_HANDLE[instance_idx_aura], true);
                        
                        // Re-apply visual settings periodically to prevent drift
                        if player_state.current_frame % 30 == 0 {
                            let settings = crate::gastly::effects::GASTLY_AURA_SETTINGS;
                            EffectModule::set_rgb(boma, SPECIAL_AURA_HANDLE[instance_idx_aura], 
                                                 settings.color_r, settings.color_g, settings.color_b);
                            EffectModule::set_alpha(boma, SPECIAL_AURA_HANDLE[instance_idx_aura], settings.alpha);
                            EffectModule::set_rate(boma, SPECIAL_AURA_HANDLE[instance_idx_aura], settings.rate);
                        }
                    } else {
                        // Respawn if effect was killed during special states
                        let handle = crate::gastly::effects::spawn_gastly_aura_direct(boma);
                        if handle != 0 {
                            SPECIAL_AURA_HANDLE[instance_idx_aura] = handle;
                            EffectModule::set_visible(boma, handle, true);
                        }
                    }
                }
            }
        } else {
            // Clean up special aura when we exit special situations
            let instance_idx_cleanup = instance_key as usize;
            if instance_idx_cleanup < 256 {
                static mut SPECIAL_AURA_SPAWNED: [bool; 256] = [false; 256];
                static mut SPECIAL_AURA_HANDLE: [u32; 256] = [0; 256];
                
                if SPECIAL_AURA_HANDLE[instance_idx_cleanup] != 0 {
                    EffectModule::kill(boma, SPECIAL_AURA_HANDLE[instance_idx_cleanup], false, true);
                    SPECIAL_AURA_HANDLE[instance_idx_cleanup] = 0;
                }
                SPECIAL_AURA_SPAWNED[instance_idx_cleanup] = false;
                
                // Clear the flag so normal aura system can take over
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
            }
        }
    }

    // Start final smash cleanup fix
    unsafe fn handle_final_smash_aggressive_fixes(
    boma: *mut BattleObjectModuleAccessor, 
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon
) {
    let current_status = StatusModule::status_kind(boma);
    let is_final_smash_flag = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    
    //  Track when we're in background-setting animations
    let current_motion = MotionModule::motion_kind(boma);
    let is_background_setting_animation = current_motion == smash::hash40("final_start_r") ||
                                         current_motion == smash::hash40("final_start_l") ||
                                         current_motion == smash::hash40("final_air_start_r") ||
                                         current_motion == smash::hash40("final_air_start_l");
    
    // TARGETED AURA KILLING during ALL final smash statuses - BUT NOT BACKGROUNDS
    if current_status == 0x1E0 ||  // FINAL
       current_status == 0x1E8 ||  // FINAL_WAIT  
       current_status == 0x1E9 {   // FINAL_END
        
        //  Only kill specific aura effects, NOT screen effects
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura_charge"), false, true);
        
        // DO NOT kill screen effects during background-setting animations
        if !is_background_setting_animation {
            // Only kill default background if we're not in the process of setting a custom one
            EffectModule::kill_kind(boma, Hash40::new("bg_purin_final"), false, false);
            EffectModule::kill_kind(boma, Hash40::new("purin_final_bg_black"), false, false);
        }
        
        if (instance_key as usize) < 256 && current_status == 0x1E0 {
        }
    }
        
    //  Only activate special forms during FINAL status (0x1E0), not FINAL_START
    if current_status == 0x1E0 && // FINAL status only
       is_final_smash_flag && 
       !player_state.is_in_final_smash_form && 
       player_state.stage == EvolutionStage::Gengar {
        
        if player_state.mega_gengar_form_active || player_state.giga_gengar_form_active {
            
            // Kill aura during activation - BUT NOT BACKGROUNDS
            EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
            EffectModule::kill_kind(boma, Hash40::new("sys_final_aura"), false, true);
            
            // Hide normal Gengar completely
            ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
            ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
            ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
            ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
            for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() {
                ModelModule::set_mesh_visibility(boma, *eye_hash, false);
            }
            
            // Hide Gastly body during special forms
            ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
            
            // Show appropriate special form
            if player_state.mega_gengar_form_active {
                ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, true);
            } else if player_state.giga_gengar_form_active {
                ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, true);
            }
            
            player_state.is_in_final_smash_form = true;
        }
    }
    
    // CONTINUOUS targeted aura killing while in final smash form - BUT NOT BACKGROUNDS
    if player_state.is_in_final_smash_form {
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura"), false, true);
    }
    
    //  Specific handling for FINAL_END status to prevent Gastly body stacking
    if current_status == 0x1E9 { // FINAL_END
        
        // AGGRESSIVELY hide Gastly body during FINAL_END
        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
        
        // Ensure special form stays visible during FINAL_END
        if player_state.is_in_final_smash_form {
            if player_state.mega_gengar_form_active {
                ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, true);
                ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
            } else if player_state.giga_gengar_form_active {
                ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, true);
                ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
            }
        }
    }

    // Handle catch effect cleanup - kill ridley_grabbing_catch when not in catch states
    let current_status_val: i32 = StatusModule::status_kind(boma);
    let is_catch_pull = current_status_val == *FIGHTER_STATUS_KIND_CATCH_PULL;
    let is_catch_wait = current_status_val == *FIGHTER_STATUS_KIND_CATCH_WAIT;

    // Clean up catch effect when not in catch_pull or catch_wait states
    if !is_catch_pull && !is_catch_wait {
        // Check if this player is in Gastly stage before killing effect
        let is_gastly_stage = player_state.stage == EvolutionStage::Gastly;
        
        if is_gastly_stage {
            EffectModule::kill_kind(boma, Hash40::new("ridley_grabbing_catch"), false, true);
        }
    }
    
    // CLEANUP: When final smash flag goes false, restore normal appearance
    if !is_final_smash_flag && player_state.is_in_final_smash_form {
        
        // Kill any remaining aura effects
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
        
        // Hide ALL special meshes
        ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
        
        // Reset flag
        player_state.is_in_final_smash_form = false;
        
        // Force normal Gengar appearance
        ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, true);
        ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, true);
        ModelModule::set_mesh_visibility(boma, *GENGAR_EYE_N, true);
        
    }
}

// Helper function to handle grab effect cleanup based on status changes
unsafe fn handle_grab_effect_cleanup(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) {
    let instance_key = get_instance_key(boma) as usize;
    if (instance_key as usize) >= 256 { return; }
    
    //  Always run cleanup for ALL stages (not just Gastly)
    // This ensures the grab effect gets killed even if you evolve while grabbing
    
    let current_status = StatusModule::status_kind(boma);
    let last_status = LAST_GRAB_STATUS[instance_key];
    let status_changed = last_status != current_status;
    
    // Kill grab effect if:
    // 1. Status changed after being in CATCH_WAIT, OR
    // 2. Status changed after being in CATCH_PULL and current status is NOT CATCH_WAIT
    if status_changed {
        let should_kill = (last_status == *FIGHTER_STATUS_KIND_CATCH_WAIT) ||
                         (last_status == *FIGHTER_STATUS_KIND_CATCH_PULL && 
                          current_status != *FIGHTER_STATUS_KIND_CATCH_WAIT);
        
        if should_kill {
            EffectModule::kill_kind(boma, Hash40::new("ridley_grabbing_catch"), false, true);
        }
    }
    
    // Update status tracking
    LAST_GRAB_STATUS[instance_key] = current_status;
}


    // End final smash cleanup fix

    if is_new_state {
        player_state.set_vanilla_expression_tracking(true);
    }

    player_state.current_frame += 1;
    player_state.manual_linking_cord_evo_attempted_this_frame = false;
    player_state.linking_cord_consumed_everstone_this_frame = false;
    

    let current_total_damage_on_self = DamageModule::damage(boma, 0);

    //  Handle training mode fixed damage properly
    static mut LAST_TRAINING_DAMAGE: [f32; 256] = [0.0; 256];

    //  Handle training mode damage detection properly
    let entry_idx = my_entry_id_u32 as usize;
    if entry_idx < 8 {
        // Debug training damage detection
        if player_state.current_frame % 60 == 0 {
        }
        
        //  Track incremental damage taken during current evolution stage
        if !player_state.is_evolving {
            let damage_since_stage_start = current_total_damage_on_self - player_state.previous_total_damage;
            
            // Only count positive damage increases (ignore healing/resets)
            if damage_since_stage_start > 0.0 {
                player_state.damage_received_this_stage += damage_since_stage_start;
                player_state.previous_total_damage = current_total_damage_on_self;
            }
            // Handle damage resets (training mode) by resetting our tracking
            else if current_total_damage_on_self < player_state.previous_total_damage * 0.5 {
                player_state.damage_received_this_stage = 0.0;
                player_state.previous_total_damage = current_total_damage_on_self;
            }
        }
        
        if (instance_key as usize) < 256 {
            LAST_TRAINING_DAMAGE[instance_key as usize] = current_total_damage_on_self;
        }
    }

    // Debug hit tracking and aggressive reset detection
    static mut LAST_HIT_COUNT: [i32; 256] = [0; 256];
    static mut LAST_HIT_RESET_FRAME: [i32; 256] = [0; 256];
    
    let entry_idx = my_entry_id_u32 as usize;
    if entry_idx < 8 {
        // IMPROVED: More aggressive hit count reset detection
        let current_damage = DamageModule::damage(boma, 0);
        let current_frame = player_state.current_frame;
        
        // More sensitive reset conditions:
        // 1. Damage resets to very low (training mode reset)
        // 2. Frame counter resets/jumps backwards significantly 
        // 3. Status shows potential reset (training reset handler was called)
        let damage_reset_detected = current_damage <= 1.0 && (instance_key as usize) < 256 && LAST_TRAINING_DAMAGE[instance_key as usize] > 5.0;
        let frame_reset_detected = (instance_key as usize) < 256 && current_frame < LAST_HIT_RESET_FRAME[instance_key as usize] - 50;
        let current_status = StatusModule::status_kind(boma);
        let status_suggests_reset = current_status == *FIGHTER_STATUS_KIND_STANDBY || 
                                   current_status == *FIGHTER_STATUS_KIND_ENTRY;
        
        //  Only reset on damage/frame resets, not status alone (status check was too aggressive)
        let should_reset_hits = (damage_reset_detected || frame_reset_detected) && 
                               player_state.hits_landed_this_stage > 0;
        
        if should_reset_hits {
            player_state.hits_landed_this_stage = 0;
            if (instance_key as usize) < 256 {
                LAST_HIT_RESET_FRAME[instance_key as usize] = current_frame;
            }
        }
        
        // Debug hit tracking changes
        if (instance_key as usize) < 256 && player_state.hits_landed_this_stage != LAST_HIT_COUNT[instance_key as usize] {
            if (instance_key as usize) < 256 {
                LAST_HIT_COUNT[instance_key as usize] = player_state.hits_landed_this_stage;
            }
        }
    }
    
    player_state.previous_total_damage = current_total_damage_on_self;

    let current_status_val: i32 = StatusModule::status_kind(boma);

    // Shield break fly effects on first frame
    static mut LAST_SHIELD_BREAK_STATUS: [i32; 256] = [-1; 256];
    let entry_id = my_entry_id_u32 as usize;
    
    if entry_id < 8 && current_status_val == *FIGHTER_STATUS_KIND_SHIELD_BREAK_FLY {
        let status_just_changed = (instance_key as usize) < 256 && LAST_SHIELD_BREAK_STATUS[instance_key as usize] != current_status_val;
        
        if status_just_changed {
            // sys_drill_smoke effect with proper scale
            macros::EFFECT(fighter, Hash40::new("sys_drill_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
            let scale_axis = Vector3f{ x: 1.0, y: 2.0, z: 1.0 };
            EffectModule::set_scale_last(fighter.module_accessor, &scale_axis);
            macros::LAST_EFFECT_SET_RATE(fighter, 0.4);
            macros::LAST_EFFECT_SET_COLOR(fighter, 0.1, 0.0, 0.1);
            macros::LAST_EFFECT_SET_ALPHA(fighter, 0.8);
            
            // rosetta_galaxyjump effect (req_follow)
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };

            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("rosetta_galaxyjump"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                3.0,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;

            EffectModule::set_rgb(boma, handle, 1.0, 1.0, 1.0);
            EffectModule::set_alpha(boma, handle, 1.0);
            
            // edge_gokumon_impact effect
            macros::EFFECT(fighter, Hash40::new("edge_gokumon_impact"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
            macros::LAST_EFFECT_SET_RATE(fighter, 0.3);
            macros::LAST_EFFECT_SET_COLOR(fighter, 1.0, 0.5, 1.0);
            macros::LAST_EFFECT_SET_ALPHA(fighter, 1.0);
        }
    }
    
    // Update status tracking for all players
    if entry_id < 8 {
        if (instance_key as usize) < 256 {
            LAST_SHIELD_BREAK_STATUS[instance_key as usize] = current_status_val;
        }
    }

    let is_dead_or_rebirth = current_status_val == *FIGHTER_STATUS_KIND_DEAD || current_status_val == *FIGHTER_STATUS_KIND_REBIRTH;


    if is_dead_or_rebirth {
               
        cleanup_all_evolution_sounds_on_death(boma);

        // Preserve shiny timer states during cleanup
        let preserve_sparkle_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);
        let preserve_effect_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        let preserve_sparkle_active = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
        let preserve_effect_active = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);

        crate::gastly::persist_sfx::cleanup_evolution_sounds_on_death(boma);
        crate::gastly::darkfx::cleanup_dark_effects_on_death(my_entry_id_u32);
        // Clean up motion-based sounds
        crate::gastly::sounds::cleanup_motion_sounds_on_death(boma);
        // Clean up universal effects for this player
        crate::gastly::effects::cleanup_player_universal_effects(instance_key);
        
        // Clean up weakened effect during death/respawn - let the visuals.rs system handle respawning
        EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_weak"), false, true);
        
        // Aggressively clean up all Gastly-related effects to prevent spam in duo matches
        let gastly_effects = [
            "sys_smash_flash",
            "sys_special_all_up", 
            "bayonetta_final_cry",
            "sys_final_aura",
            "sys_final_aura_charge", 
            "sys_final_aura2",
            "mewtwo_shadowball_main",
            "mewtwo_shadowball_hold",
            "sys_speedbooster",
            "purin_appeal_lw"
        ];
        
        for effect_name in &gastly_effects {
            EffectModule::kill_kind(boma, Hash40::new(effect_name), false, true);
        }
        
        // Reset weakened state tracking so visuals.rs can properly handle respawn
        if my_entry_id_u32 < 8 {
            unsafe {
                crate::gastly::visuals::emergency_cleanup_weakened(my_entry_id_u32);
            }
        }
        
        // End of weakened state cleanup

        if player_state.is_evolving {
            macros::COL_NORMAL(fighter);
        }
        if player_state.is_in_final_smash_form {
            handle_final_smash_model_swap(boma, player_state);
        }

        deactivate_all_pos_sensitive_icons(boma, player_state);
        // Force complete state reset on death/rebirth with proper synchronization
        player_state.stage = EvolutionStage::Gastly;
        player_state.evolution_target_stage = EvolutionStage::Gastly;
        player_state.is_evolving = false;
        player_state.evolution_timer = 0;
        player_state.damage_received_this_stage = 0.0;
        player_state.hits_landed_this_stage = 0;
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
        player_state.evo_attempt_delay_hits_penalty = 0;
        player_state.previous_total_damage = 0.0;
        player_state.reset_evo_readiness_icons();
        
        // Ensure hit requirements are synchronized with Gastly stage after death
        // (prevents hit-count readiness desync after multiple deaths)
        
        // ADDITIONAL PROTECTION: Force reset hit tracking arrays to prevent stale data
        if instance_key < 256 {
            LAST_HIT_COUNT[instance_key as usize] = 0;
            LAST_HIT_RESET_FRAME[instance_key as usize] = player_state.current_frame;
        }
        
        // Call the reset function after manual reset
        player_state.full_reset_on_respawn(boma);
        
        // AGGRESSIVE PENALTY RESET: Force zero penalties after death, especially for multiple deaths
        player_state.evo_attempt_delay_hits_penalty = 0;
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
        
        // ADDITIONAL PROTECTION: Reset any stale penalty data in static arrays
        if instance_key < 256 {
            // Clear any penalty-related data that might persist
            // This addresses the "50-60 hits after multiple deaths" issue
        }
        
        // Clean up UI state on death AFTER player state reset to prevent override
        reset_ui_state_on_death(my_entry_id_u32);

        ModelModule::set_mesh_visibility(boma, *LINKING_CORD_ICON, false);
        ModelModule::set_mesh_visibility(boma, *EVERSTONE_ICON, false);
        ModelModule::set_mesh_visibility(boma, *EVERSTONE_X_ICON, false);
        ModelModule::set_mesh_visibility(boma, *GENGARITE_ICON, false);
        ModelModule::set_mesh_visibility(boma, *DYNAMAX_ICON, false);
        ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);

        for vanilla_eye in PURIN_VANILLA_EYES_TO_HIDE.iter() {
            ModelModule::set_mesh_visibility(boma, *vanilla_eye, false);
        }

        update_body_and_unique_parts_visibility_with_enforcement(boma, EvolutionStage::Gastly, player_state);
        set_active_eye_mesh(boma, player_state, None);
        // Don't return early - allow healing detection to continue even during death states
        // return;
    }

    for vanilla_eye in PURIN_VANILLA_EYES_TO_HIDE.iter() {
        ModelModule::set_mesh_visibility(boma, *vanilla_eye, false);
    }

    handle_grab_effect_cleanup(boma, player_state);

    handle_final_smash_aggressive_fixes(boma, player_state, fighter);
    
    handle_icon_toggles_and_effects(boma, player_state);
    
    // Handle readiness icons BEFORE evolution process
    handle_evolution_readiness_icons(boma, player_state, fighter);

    // EVERSTONE ICON FIX HERE:
    if player_state.everstone_icon_active || player_state.everstone_x_icon_active {
        // Force hide readiness icons when everstone is active
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);
    }
    
    // Check for final smash cancellation during evolution
    if player_state.is_evolving {
        // Only cancel if actually entering FINAL status (when final smash is used)
        let is_using_final_smash = current_status_val == 0x1E0; // FINAL status only
        
        // Cancel evolution if final smash is actually used
        if is_using_final_smash {
            player_state.cancel_evolution(fighter);
            // Don't return here - let the rest of the frame continue processing
        }
    }

    detect_healing_events(boma, player_state);

    // Evolution process comes AFTER readiness icons
    handle_evolution_process(fighter, player_state);

    // Handle persistent sounds AFTER evolution process but BEFORE effects
    handle_persistent_looping_sounds(boma, player_state, fighter);

    // Handle effects AFTER evolution process but BEFORE advance_evolution_animation
    // This ensures status change detection works properly
    handle_gastly_effects(boma, player_state, fighter);

    // Handle evolution animation advancement
    if player_state.is_evolving {
        advance_evolution_animation(fighter, player_state);
    }

    if !player_state.is_evolving && !player_state.is_in_final_smash_form {
        handle_final_smash_model_swap(boma, player_state);
    }

    if !player_state.is_evolving && !player_state.is_in_final_smash_form {
        let mut expression_override: Option<PhxHash40> = None;

        // Handle win_3 motion blink override first
        let is_win_3_blink = handle_win_3_blink(boma, player_state);

        //  Check for down special blink override
        let is_down_special_blink = if !is_win_3_blink {
            handle_down_special_blink(boma, player_state)
        } else {
            false
        };

        if !is_down_special_blink && !is_win_3_blink {
            // Only do normal expression processing if not in down special blink
            if let Some(vanilla_detected_expression) = player_state.detect_vanilla_expression(boma) {
                expression_override = Some(vanilla_detected_expression);
            } else {
                player_state.blink_timer -= 1;
                if player_state.blink_timer <= 0 {
                    player_state.advance_blink_phase();
                }
            }
            
            //  Only call set_active_eye_mesh if NOT in down special blink
            set_active_eye_mesh(boma, player_state, expression_override);
        }
        // If is_down_special_blink is true, we DON'T call set_active_eye_mesh
        // because handle_down_special_blink already set the correct mesh
    }

    // UI management is now handled globally above to detect character switches

    // Handle shiny effect timers (protected from cleanup interference)
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE) && !is_dead_or_rebirth {
        let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        let new_timer = timer + 1.0;
        WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_EFFECT_TIMER);
        
        if new_timer >= 90.0 {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_EFFECT_ACTIVE);
            EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_happy_light"), false, true);
        }
    }

    // Handle shiny sparkle sound duration (102 frames) - protected from cleanup
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE) && !is_dead_or_rebirth {
        // Debug: Check if flag was just set this frame
        let sparkle_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);
        if sparkle_timer == 0.0 {
        }
        let sparkle_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);
        let new_sparkle_timer = sparkle_timer + 1.0;
        WorkModule::set_float(boma, new_sparkle_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_SHINY_SPARKLE_TIMER);
        // Debug: Log timer progress every 30 frames
        if new_sparkle_timer as i32 % 30 == 0 {
        }
        
        if new_sparkle_timer >= 102.0 {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_SHINY_SPARKLE_ACTIVE);
            SoundModule::stop_se(boma, Hash40::new("shiny_sparkle"), 0);
        }
    }

    // Handle shiny effects
    handle_shiny_effects(boma, player_state, fighter, current_status_val, player_state.current_frame);

    // Handle delayed cry sounds for shiny pokemon
    if player_state.delayed_cry_timer > 0 {
        player_state.delayed_cry_timer -= 1;
        if player_state.delayed_cry_timer == 0 && !player_state.delayed_cry_sound.is_empty() {
            let cry_handle = SoundModule::play_se(
                boma,
                Hash40::new(&player_state.delayed_cry_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, cry_handle as i32, 2.5, 0);
            
            // Track cry for UI cutins
            let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
            crate::gastly::ui_management::track_cry_sound_playback(
                Hash40::new(&player_state.delayed_cry_sound),
                player_state.current_frame,
                entry_id
            );
            
            player_state.delayed_cry_sound = String::new();
        }
    }
}

unsafe fn reset_all_match_tracking_by_instance_key(instance_key: usize, current_frame: i32) {
    if instance_key >= 256 { return; }

    
    // Check if we already have heal data stored - don't overwrite it
    let (existing_heal_amount, existing_heal_frame) = HEAL_DETECTED[instance_key];
    if existing_heal_amount != 0.0 && existing_heal_frame > 0 {
        // Only reset damage tracker, preserve heal data
        DAMAGE_TRACKER[instance_key] = (0.0, -200);
        return;
    }
    
    // Before resetting, check if this was a heal to 0%
    let (last_damage, _) = DAMAGE_TRACKER[instance_key];
    if last_damage >= 15.0 {
        
        // PROTECTION: Don't detect healing right after rebirth exit
        // This prevents healing from being detected when exiting rebirth
        let frames_since_rebirth_exit = current_frame - LAST_REBIRTH_EXIT_FRAME[instance_key];
        if frames_since_rebirth_exit >= 0 && frames_since_rebirth_exit <= 10 {
            DAMAGE_TRACKER[instance_key] = (0.0, -200);
            HEAL_DETECTED[instance_key] = (0.0, -200);
            return;
        }
        
        // Use actual current frame to ensure proper timing
        let heal_frame = current_frame;
        
        // Check for G_RESTORE: heal from >=35% to zero percent
        if last_damage >= 35.0 {
            HEAL_DETECTED[instance_key] = (-last_damage, heal_frame);
        }
        // Check for G_POTION: heal from <35% to zero percent  
        else {
            HEAL_DETECTED[instance_key] = (last_damage, heal_frame);
        }
        
        // Don't reset HEAL_DETECTED in this case - let the sound system process it
        DAMAGE_TRACKER[instance_key] = (0.0, -200);
        return;
    }
    
    // Reset damage tracking (this prevents false heal detection)
    DAMAGE_TRACKER[instance_key] = (0.0, -200);
    HEAL_DETECTED[instance_key] = (0.0, -200);
    
}

unsafe fn detect_new_training_session_for_marked_costumes(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    entry_id: u32
) -> bool {
    let instance_key = get_instance_key(boma);
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    let is_marked_costume = if color_id < 256 {
        crate::MARKED_COLORS[color_id]
    } else {
        false
    };
    
    if !is_marked_costume {
        return false;
    }
    
    static mut LAST_SESSION_DAMAGE: [f32; 256] = [0.0; 256];
    static mut LAST_SESSION_FRAME: [i32; 256] = [0; 256];
    static mut SESSION_RESET_COOLDOWN: [i32; 256] = [0; 256];
    
    let entry_idx = entry_id as usize;
    if entry_idx >= 8 { return false; }
    
    let current_damage = DamageModule::damage(boma, 0);
    let current_frame = player_state.current_frame;
    
    // Decrement cooldown
    if (instance_key as usize) < 256 && SESSION_RESET_COOLDOWN[instance_key as usize] > 0 {
        if (instance_key as usize) < 256 {
            SESSION_RESET_COOLDOWN[instance_key as usize] -= 1;
        }
        return false;
    }
    
    // Detect new session: damage reset to 0 AND frame counter reset/jumped backwards
    let damage_reset = current_damage <= 0.1 && (instance_key as usize) < 256 && LAST_SESSION_DAMAGE[instance_key as usize] > 5.0;
    let frame_reset = (instance_key as usize) < 256 && (current_frame < LAST_SESSION_FRAME[instance_key as usize] - 100 || 
                 (current_frame < 30 && LAST_SESSION_FRAME[instance_key as usize] > 100));
    
    // Don't reset during results screen
    let current_status = StatusModule::status_kind(boma);
    let is_results_screen = current_status == *FIGHTER_STATUS_KIND_WIN ||
                        current_status == *FIGHTER_STATUS_KIND_LOSE ||
                        current_status == 0x107 ||
                        (current_status >= 0x190 && current_status <= 0x1DC);

    if (damage_reset || frame_reset) && !is_results_screen {
        
        //  Always reset evolution tracking on session reset, regardless of stage
        
        // Reset ALL evolution tracking on new session (regardless of current stage)
        player_state.damage_received_this_stage = 0.0;
        player_state.hits_landed_this_stage = 0;
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
        player_state.evo_attempt_delay_hits_penalty = 0;
        player_state.previous_total_damage = 0.0;
        player_state.reset_evo_readiness_icons();
        
        // Force reset to Gastly for marked costumes if not already Gastly
        if player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
            player_state.full_reset_on_respawn(boma);
            player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;

            player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
            
            if (instance_key as usize) < 256 {
                SESSION_RESET_COOLDOWN[instance_key as usize] = 300; // 5 second cooldown
                LAST_SESSION_DAMAGE[instance_key as usize] = current_damage;
                LAST_SESSION_FRAME[instance_key as usize] = current_frame;
            }
            return true;
        }
    }
    
    // Update tracking
    if (instance_key as usize) < 256 {
        LAST_SESSION_DAMAGE[instance_key as usize] = current_damage;
        LAST_SESSION_FRAME[instance_key as usize] = current_frame;
    }
    
    false
}

// Enhanced heal detection function - call this BEFORE handle_persistent_looping_sounds
unsafe fn detect_healing_events(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState
) {
    static mut LAST_CALLBACK_DAMAGE: [f32; 256] = [0.0; 256];
    
    let instance_key = get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    let current_status = StatusModule::status_kind(boma);
    let current_frame = player_state.current_frame;
    let current_damage = DamageModule::damage(boma, 0);
    
    // Check for g_restore using backup damage tracker
    let last_callback_damage = LAST_CALLBACK_DAMAGE[instance_key];
    if current_damage <= 0.1 && last_callback_damage >= 35.0 {
        HEAL_DETECTED[instance_key] = (-last_callback_damage, current_frame);
        LAST_CALLBACK_DAMAGE[instance_key] = current_damage; // Update for next time
        return;
    }
    
    // Update backup tracker
    LAST_CALLBACK_DAMAGE[instance_key] = current_damage;
    
    // ENHANCED: More aggressive exclusion including transitions (but allow DEAD/REBIRTH for g_restore)
    let excluded_statuses = [
        // *FIGHTER_STATUS_KIND_DEAD,      // 0xB5 - Allow for g_restore detection
        // *FIGHTER_STATUS_KIND_REBIRTH,   // 0xB6 - Allow for g_restore detection  
        *FIGHTER_STATUS_KIND_STANDBY,   // 0x1D6
        *FIGHTER_STATUS_KIND_ENTRY,     // 0x1D9
    ];
    
    // Also check if we recently died (within last 240 frames = 4 seconds)
    
    if excluded_statuses.contains(&current_status) {
        HEAL_DETECTED[instance_key] = (0.0, -200);
        DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
        // Track when we were in death/rebirth
        if current_status == *FIGHTER_STATUS_KIND_DEAD || current_status == *FIGHTER_STATUS_KIND_REBIRTH {
            LAST_DEATH_FRAME[instance_key] = current_frame;
        }
        return;
    }
    
    // Don't detect healing if we recently died/respawned (prevent false healing detection during respawn)
    let frames_since_death = current_frame - LAST_DEATH_FRAME[instance_key];
    
    // Special case: If current_frame is very low (like 1) and we have potential g_restore, don't block
    let is_potential_g_restore_frame_reset = current_damage <= 0.1 && current_frame <= 10;
    
    if frames_since_death >= 0 && frames_since_death <= 360 && !is_potential_g_restore_frame_reset { // Increased to 6 seconds
        HEAL_DETECTED[instance_key] = (0.0, -200);
        DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
        return;
    }
    
    // Enhanced rebirth exit detection
    let last_status = LAST_STATUS[instance_key];
    LAST_STATUS[instance_key] = current_status;
    
    // If we just exited rebirth status (including death during rebirth), record the frame and prevent healing detection
    if last_status == *FIGHTER_STATUS_KIND_REBIRTH && current_status != *FIGHTER_STATUS_KIND_REBIRTH {
        LAST_REBIRTH_EXIT_FRAME[instance_key] = current_frame;
        HEAL_DETECTED[instance_key] = (0.0, -200);
        DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
        return;
    }
    
    // Also track death during rebirth as rebirth exit
    if last_status == *FIGHTER_STATUS_KIND_REBIRTH && current_status == *FIGHTER_STATUS_KIND_DEAD {
        LAST_REBIRTH_EXIT_FRAME[instance_key] = current_frame;
        LAST_DEATH_FRAME[instance_key] = current_frame; // Also update death frame
        HEAL_DETECTED[instance_key] = (0.0, -200);
        DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
        return;
    }
    
    // Block healing detection for 180 frames (3 seconds) after exiting rebirth
    let frames_since_rebirth_exit = current_frame - LAST_REBIRTH_EXIT_FRAME[instance_key];
    if frames_since_rebirth_exit >= 0 && frames_since_rebirth_exit <= 180 {
        HEAL_DETECTED[instance_key] = (0.0, -200);
        DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
        return;
    }
    
    // No early frame skip - healing items can reset frame counter to 1
    
    let (last_damage, last_frame) = DAMAGE_TRACKER[instance_key];
    
    // Only proceed if we have valid previous data (handle frame resets) OR if we detect a major damage drop
    let major_damage_drop = current_damage <= 0.1 && last_damage > 15.0;  // Changed to <= 0.1 to match g_restore logic
    
    // Special case: If we're at frame 1 with 0% damage, check if this could be a g_restore scenario
    // In training mode, healing can reset frames to 1, so we need to allow this case
    let potential_training_heal = current_frame <= 5 && current_damage <= 0.1 && last_frame < 0;
    
    let should_check = ((current_frame - last_frame >= 1 || current_frame < last_frame) && last_frame >= 0 && last_damage >= 0.0) || major_damage_drop || potential_training_heal;  // Changed > 0.0 to >= 0.0
    
    if should_check {
        let damage_change = current_damage - last_damage;
        
        // Basic validation for legitimate heals (not from match transitions)
        let is_reasonable_heal = damage_change <= -15.0 && 
                                damage_change >= -200.0 && // Not too huge (max 200% heal)
                                current_frame - last_frame < 600; // Within 10 seconds
        
        // Special case: Always allow potential g_restore heals (35%+ to 0%)
        let is_potential_g_restore = current_damage <= 0.1 && last_damage >= 35.0;
        
        if !is_reasonable_heal && !is_potential_g_restore {
            DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
            return;
        }
        
        // ADDITIONAL PROTECTION: Don't detect healing after recent rebirth exit (but allow g_restore)
        let frames_since_rebirth_exit = current_frame - LAST_REBIRTH_EXIT_FRAME[instance_key];
        if frames_since_rebirth_exit >= 0 && frames_since_rebirth_exit <= 180 && !is_potential_g_restore {
            DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
            return; // Skip healing detection after rebirth exit (except for g_restore)
        }
        
        // Don't detect healing if we recently died/respawned (but allow g_restore)
        let frames_since_death = current_frame - LAST_DEATH_FRAME[instance_key];
        if frames_since_death >= 0 && frames_since_death <= 360 && !is_potential_g_restore {
            DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
            return; // Skip healing detection after death (except for g_restore)
        }
        
        // Check for G_RESTORE: heal from >=35% to zero percent (has priority over G_POTION)
        if current_damage <= 0.1 && last_damage >= 35.0 {
            let heal_amount = last_damage;
            HEAL_DETECTED[instance_key] = (-heal_amount, current_frame); // Negative = g_restore
        }
        // Check for G_POTION: significant heal (≥15%) from any starting damage, EXCEPT when healing from ≥35% to 0%
        else if damage_change <= -15.0 && !(current_damage <= 0.1 && last_damage >= 35.0) {
            let heal_amount = damage_change.abs();
            HEAL_DETECTED[instance_key] = (heal_amount, current_frame); // Positive = g_potion
        }
    }
    
    // Always update tracker
    DAMAGE_TRACKER[instance_key] = (current_damage, current_frame);
}

// Helper functions for resetting damage and heal tracking during training mode resets
pub unsafe fn reset_damage_tracker_for_entry(boma: *mut BattleObjectModuleAccessor) {
    let instance_key = get_instance_key(boma) as usize;
    if instance_key < 256 {
        DAMAGE_TRACKER[instance_key] = (0.0, -200);
    }
}

pub unsafe fn reset_heal_tracker_for_entry(boma: *mut BattleObjectModuleAccessor) {
    let instance_key = get_instance_key(boma) as usize;
    if instance_key < 256 {
        HEAL_DETECTED[instance_key] = (0.0, -200);
        // Also reset the death tracking to prevent false healing detection
        LAST_DEATH_FRAME[instance_key] = -300;
        // Reset status tracking to prevent false rebirth exit detection
        LAST_STATUS[instance_key] = -1;
        LAST_REBIRTH_EXIT_FRAME[instance_key] = -300;
    }
}

unsafe fn handle_persistent_looping_sounds(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    _fighter: &mut L2CFighterCommon
) {
    // Store sound handles for proper management
    static mut EVOLVING_SOUND_HANDLE: [u32; 256] = [0; 256];
    static mut EVOLVE_SS_SOUND_HANDLE: [u32; 256] = [0; 256];
    static mut SHADOWBALL_CHARGE_HANDLE: [u32; 256] = [0; 256];
    static mut G_GRAB_BURN_HANDLE: [u32; 256] = [0; 256];   
    static mut MEGASYMBOL_HANDLE: [u32; 256] = [0; 256];       
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if (instance_key as usize) >= 256 { return; }
    
    // Track evolution state for proper sound transitions
    static mut LAST_EVOLUTION_STATE: [bool; 256] = [false; 256];
    let evolution_just_started = !LAST_EVOLUTION_STATE[instance_key] && player_state.is_evolving;
    LAST_EVOLUTION_STATE[instance_key] = player_state.is_evolving;
    
    // ===== EVOLVING SOUND (HIGHEST PRIORITY) =====
    //  Update timer regardless of is_evolving state
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE) {
        // Always update timer when sound is active, regardless of evolution state
        let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
        let new_timer = timer + 1.0;
        WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
        
        // Check if sound has reached 459 frames - MANUALLY stop the looping sound
        if new_timer >= 459.0 {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
            
            // Must manually stop the looping sound
            if EVOLVING_SOUND_HANDLE[instance_key] != 0 {
                SoundModule::stop_se_handle(boma, EVOLVING_SOUND_HANDLE[instance_key] as i32, 0);
                EVOLVING_SOUND_HANDLE[instance_key] = 0;
            }
            SoundModule::stop_se(boma, Hash40::new("evolving"), 0);
            
        }
    }
    
    if player_state.is_evolving {
        if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE) {
            // When evolution starts, IMMEDIATELY stop evolve_ss
            if evolution_just_started {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
                if EVOLVE_SS_SOUND_HANDLE[instance_key] != 0 {
                    SoundModule::stop_se_handle(boma, EVOLVE_SS_SOUND_HANDLE[instance_key] as i32, 0);
                    EVOLVE_SS_SOUND_HANDLE[instance_key] = 0;
                }
                SoundModule::stop_se(boma, Hash40::new("evolve_ss"), 0);
            }
            
            // Start evolving sound ONCE per evolution
            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
            WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
            
            // ANTI-INTERRUPTION: Use play_se with loop=true for persistence, but manage manually
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("evolving"),
                true, // Loop=true for anti-interruption, but we'll stop it manually at 459 frames
                false, false, false,
                smash::app::enSEType(0)
            ) as u32;
            
            EVOLVING_SOUND_HANDLE[instance_key] = sfx_handle;
            SoundModule::set_se_vol(boma, sfx_handle as i32, 1.5, 0);
            
        }
    } else {
        // Only force stop if evolution was cancelled (not completed)
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE) && 
           player_state.evolution_just_cancelled_this_frame {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
            
            if EVOLVING_SOUND_HANDLE[instance_key] != 0 {
                SoundModule::stop_se_handle(boma, EVOLVING_SOUND_HANDLE[instance_key] as i32, 0);
                EVOLVING_SOUND_HANDLE[instance_key] = 0;
            }
            SoundModule::stop_se(boma, Hash40::new("evolving"), 0);
            
        }
    }
    
    // ===== EVOLVE_SS SOUND (READINESS ICONS) =====
    // Only play during readiness icons AND not during evolution
    let ss_icon_active = player_state.dmg_ss_icon_display_timer > 0;
    let se_icon_active = player_state.dmg_se_icon_display_timer > 0;
    let should_play_ss = (ss_icon_active || se_icon_active) && !player_state.is_evolving;
    
    if should_play_ss {
        if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE) {
            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
            WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
            
            // Use regular play_se with loop=true for anti-interruption
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("evolve_ss"),
                true, // Loop=true for anti-interruption against jumps
                false, false, false,
                smash::app::enSEType(0)
            ) as u32;
            
            EVOLVE_SS_SOUND_HANDLE[instance_key] = sfx_handle;
            SoundModule::set_se_vol(boma, sfx_handle as i32, 1.8, 0);
        }
                
        // Update timer
        let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
        WorkModule::set_float(boma, timer + 1.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
        
    } else {
        // Stop evolve_ss when conditions are no longer met
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE) {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
            
            if EVOLVE_SS_SOUND_HANDLE[instance_key] != 0 {
                SoundModule::stop_se_handle(boma, EVOLVE_SS_SOUND_HANDLE[instance_key] as i32, 0);
                EVOLVE_SS_SOUND_HANDLE[instance_key] = 0;
            }
            SoundModule::stop_se(boma, Hash40::new("evolve_ss"), 0);
            
            let reason = if player_state.is_evolving { "evolution started" } else { "icons ended" };
        }
    }
    
    // ===== SHADOWBALL CHARGE SOUND =====
    let current_status = StatusModule::status_kind(boma);
    let is_charging = current_status == PURIN_SPECIAL_N_HOLD || current_status == PURIN_SPECIAL_N_HOLD_MAX;
    
    if is_charging {
        if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE) {
            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE);
            
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("g_shadowball_charge"),
                true, // Loop for persistence
                false, false, false,
                smash::app::enSEType(0)
            ) as u32;
            
            SHADOWBALL_CHARGE_HANDLE[instance_key] = sfx_handle;
            SoundModule::set_se_vol(boma, sfx_handle as i32, 1.0, 0);
        }
    } else {
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE) {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE);
            
            if SHADOWBALL_CHARGE_HANDLE[instance_key] != 0 {
                SoundModule::stop_se_handle(boma, SHADOWBALL_CHARGE_HANDLE[instance_key] as i32, 0);
                SHADOWBALL_CHARGE_HANDLE[instance_key] = 0;
            }
            SoundModule::stop_se(boma, Hash40::new("g_shadowball_charge"), 0);
        }
    }
    
    // Enhanced debug - only when sound is active to reduce spam
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE) {
        let evolving_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
        if player_state.current_frame % 120 == 0 { // Reduced frequency
        }
    }
    // ===== G_SHADOWBALL SOUND (when mewtwo_shadowball effect is visible) =====
    // Check if we should play g_shadowball sound based on shadowball state
    let shadowball_state = crate::gastly::visuals::detect_shadowball_hitbox_state(boma, player_state);
    let should_play_shadowball = match shadowball_state {
        crate::gastly::visuals::ShadowballState::ActiveFrameBased |
        crate::gastly::visuals::ShadowballState::ActiveWithHitbox |
        crate::gastly::visuals::ShadowballState::ChargedRolloutWithHitbox |
        crate::gastly::visuals::ShadowballState::RegularRolloutWithHitbox |
        crate::gastly::visuals::ShadowballState::AirToGroundRolloutWithHitbox => true,
        _ => false,
    };
    
    if should_play_shadowball {
        if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE) {
            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE);
            
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("g_shadowball"),
                true, // Loop for persistence
                false, false, false,
                smash::app::enSEType(0)
            ) as u32;
            
            SoundModule::set_se_vol(boma, sfx_handle as i32, 2.5, 0);
        }
    } else {
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE) {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_ACTIVE);
            SoundModule::stop_se(boma, Hash40::new("g_shadowball"), 0);
        }
    }

    // ===== SPECIAL N SOUND EFFECTS =====
    let current_status = StatusModule::status_kind(boma);
    
    // Track status changes for one-shot sound detection
    static mut LAST_SPECIAL_N_STATUS: [i32; 256] = [-1; 256];
    static mut REACHED_HOLD_MAX: [bool; 256] = [false; 256];
    static mut HAD_TURN_STATUS: [bool; 256] = [false; 256];
    
    let instance_key = get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    let status_just_changed = LAST_SPECIAL_N_STATUS[instance_key] != current_status;
    
    // Track if player went through turn status
    if current_status == PURIN_SPECIAL_N_TURN {
        HAD_TURN_STATUS[instance_key] = true;
    }
    // 1. SPECIAL_N_CHARGE_MAX sound when entering HOLD_MAX status
    if current_status == PURIN_SPECIAL_N_HOLD_MAX && status_just_changed {
        // Mark that this player reached hold max
        REACHED_HOLD_MAX[instance_key] = true;
        
        let sfx_handle = SoundModule::play_se(
            boma,
            Hash40::new("special_n_charge_max"),
            false, // One-shot
            false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, sfx_handle as i32, 0.7, 0);
    }
    
    // 2. SPECIAL_N_RELEASE sounds when entering roll/roll_air status
    // BUT ONLY if player did NOT go through turn status
    // AND did NOT transition from roll to roll_air or vice versa
    // AND shadowball effect exists (model is invisible)
    let is_roll_status = current_status == PURIN_SPECIAL_N_ROLL || 
                        current_status == PURIN_SPECIAL_N_ROLL_AIR;
    
    let is_roll_to_roll_transition = (current_status == PURIN_SPECIAL_N_ROLL && LAST_SPECIAL_N_STATUS[instance_key] == PURIN_SPECIAL_N_ROLL_AIR) ||
                                    (current_status == PURIN_SPECIAL_N_ROLL_AIR && LAST_SPECIAL_N_STATUS[instance_key] == PURIN_SPECIAL_N_ROLL);
    
    // Check if shadowball effect should be active (model invisible)
    let shadowball_state = crate::gastly::visuals::detect_shadowball_hitbox_state(boma, player_state);
    let has_shadowball_effect = match shadowball_state {
        crate::gastly::visuals::ShadowballState::ChargedRolloutWithHitbox |
        crate::gastly::visuals::ShadowballState::RegularRolloutWithHitbox |
        crate::gastly::visuals::ShadowballState::AirToGroundRolloutWithHitbox => true,
        // SPECIAL CASE: For air roll, also check if we were sufficiently charged (invisible rollout)
        crate::gastly::visuals::ShadowballState::ChargedRollout => {
            // Air roll should be allowed if sufficiently charged, even without immediate hitbox
            current_status == PURIN_SPECIAL_N_ROLL_AIR
        },
        _ => false,
    };
    
    // Additional check: Air roll should also work if player was sufficiently charged from hold status
    let has_shadowball_effect_for_air = has_shadowball_effect || 
        (current_status == PURIN_SPECIAL_N_ROLL_AIR && player_state.shadowball_was_sufficiently_charged);
    
    if is_roll_status && status_just_changed && !HAD_TURN_STATUS[instance_key] && !is_roll_to_roll_transition && has_shadowball_effect_for_air {
        if REACHED_HOLD_MAX[instance_key] {
            // Player reached hold max, use charge max release sound
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("special_n_charge_max_release"),
                false, // One-shot
                false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, sfx_handle as i32, 0.6, 0);
        } else {
            // Player did NOT reach hold max, use regular release sound
            let sfx_handle = SoundModule::play_se(
                boma,
                Hash40::new("special_n_regular_release"),
                false, // One-shot
                false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, sfx_handle as i32, 0.3, 0);
        }
    } else if is_roll_status && status_just_changed && HAD_TURN_STATUS[instance_key] {
    } else if is_roll_status && status_just_changed && is_roll_to_roll_transition {
    } else if is_roll_status && status_just_changed && !has_shadowball_effect_for_air {
    }
    
    // Reset flags when completely out of special N sequence
    let is_any_special_n_status = current_status == PURIN_SPECIAL_N_HOLD ||
                                 current_status == PURIN_SPECIAL_N_HOLD_MAX ||
                                 current_status == PURIN_SPECIAL_N_ROLL ||
                                 current_status == PURIN_SPECIAL_N_ROLL_AIR ||
                                 current_status == PURIN_SPECIAL_N_TURN ||
                                 current_status == PURIN_SPECIAL_N_END;
    
    if !is_any_special_n_status && (REACHED_HOLD_MAX[instance_key] || HAD_TURN_STATUS[instance_key]) {
        REACHED_HOLD_MAX[instance_key] = false;
        HAD_TURN_STATUS[instance_key] = false;
    }
    
    // Update status tracking
    LAST_SPECIAL_N_STATUS[instance_key] = current_status;

    // ===== G_POTION & G_RESTORE SOUND (HEALING DETECTION) =====
    let instance_key = get_instance_key(boma) as usize;
    if instance_key < 256 {

        let current_status = StatusModule::status_kind(boma);
    
        // Stop healing sounds immediately if in death/rebirth/entry statuses
    let excluded_statuses = [
        *FIGHTER_STATUS_KIND_DEAD,      // 0xB5
        *FIGHTER_STATUS_KIND_REBIRTH,   // 0xB6  
        *FIGHTER_STATUS_KIND_STANDBY,   // 0x1D6
        *FIGHTER_STATUS_KIND_ENTRY,     // 0x1D9
    ];
    
    if excluded_statuses.contains(&current_status) {
        // Force stop both healing sounds during excluded statuses
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE) {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
            SoundModule::stop_se(boma, Hash40::new("g_restore"), 0);
        }
        
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE) {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
            SoundModule::stop_se(boma, Hash40::new("g_potion"), 0);
        }
        
        // Clear heal tracker
        HEAL_DETECTED[instance_key] = (0.0, -200);
        return; // Skip rest of healing logic during excluded statuses
    }

    // Block healing sounds for 180 frames (3 seconds) after exiting rebirth
    let frames_since_rebirth_exit = player_state.current_frame - LAST_REBIRTH_EXIT_FRAME[instance_key];
    if frames_since_rebirth_exit >= 0 && frames_since_rebirth_exit <= 180 {
        HEAL_DETECTED[instance_key] = (0.0, -200);
        return; // Skip healing sound logic after rebirth exit
    }

        // Check if we have a recent significant heal (within last 120 frames)
        let (heal_amount, heal_frame) = HEAL_DETECTED[instance_key];
        let frames_since_heal = player_state.current_frame - heal_frame;
        
        // ADDITIONAL SAFETY CHECK: Don't process healing that was detected before or during rebirth
        // This prevents stale healing data from being processed after rebirth exit
        if heal_frame <= LAST_REBIRTH_EXIT_FRAME[instance_key] {
            HEAL_DETECTED[instance_key] = (0.0, -200);
            return; // Skip processing stale healing data from before rebirth exit
        }
        
        
        let is_g_restore = heal_amount < 0.0; // Negative = g_restore
        let actual_heal_amount = heal_amount.abs();
        
        let has_recent_g_restore = is_g_restore && actual_heal_amount >= 35.0 && frames_since_heal <= 120 && frames_since_heal >= 0;
        let has_recent_g_potion = !is_g_restore && actual_heal_amount >= 15.0 && frames_since_heal <= 120 && frames_since_heal >= 0;
        
        
        // G_RESTORE has priority over G_POTION
        if has_recent_g_restore {
            // Stop g_potion if it's playing
            if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE) {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
                SoundModule::stop_se(boma, Hash40::new("g_potion"), 0);
            }
            
            if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE) {
                // STOP VANILLA HEAL SOUND FIRST
                SoundModule::stop_se(boma, Hash40::new("se_common_lifeup"), 0);
                
                WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
                WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
                
                let sfx_handle = SoundModule::play_se(
                    boma,
                    Hash40::new("g_restore"),
                    true, // Loop for persistence during 105 frame duration
                    false, false, false,
                    smash::app::enSEType(0)
                ) as u32;
                
                SoundModule::set_se_vol(boma, sfx_handle as i32, 1.5, 0);
                
                // Clear the heal tracker since we've processed it
                HEAL_DETECTED[instance_key] = (0.0, -200);
            }
        }
        // G_POTION (only if g_restore is not active)
        else if has_recent_g_potion && !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE) {
            if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE) {
                // STOP VANILLA HEAL SOUND FIRST
                SoundModule::stop_se(boma, Hash40::new("se_common_lifeup"), 0);
                
                WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
                WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
                
                let sfx_handle = SoundModule::play_se(
                    boma,
                    Hash40::new("g_potion"),
                    true, // Loop for persistence during 50 frame duration
                    false, false, false,
                    smash::app::enSEType(0)
                ) as u32;
                
                SoundModule::set_se_vol(boma, sfx_handle as i32, 1.5, 0);
                
                // Clear the heal tracker since we've processed it
                HEAL_DETECTED[instance_key] = (0.0, -200);
            }
        }
        
        // Update G_RESTORE timer and auto-stop after 105 frames
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE) {
            let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
            let new_timer = timer + 1.0;
            WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
            
            // STOP VANILLA HEAL SOUND DURING G_RESTORE PLAYBACK
            SoundModule::stop_se(boma, Hash40::new("se_common_lifeup"), 0);
            
            // Stop after 105 frames
            if new_timer >= 105.0 {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
                SoundModule::stop_se(boma, Hash40::new("g_restore"), 0);
            }
        }
        
        // Update G_POTION timer and auto-stop after 50 frames
        if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE) {
            let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
            let new_timer = timer + 1.0;
            WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
            
            // STOP VANILLA HEAL SOUND DURING G_POTION PLAYBACK
            SoundModule::stop_se(boma, Hash40::new("se_common_lifeup"), 0);
            
            // Stop after 50 frames
            if new_timer >= 50.0 {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
                SoundModule::stop_se(boma, Hash40::new("g_potion"), 0);
            }
        }
        
        // Clear old heal data if it's too old
        if frames_since_heal > 120 {
            HEAL_DETECTED[instance_key] = (0.0, -200);
        }
    }

        // ===== G_FURAFURA SOUND (WITH VANILLA MUTING) =====
        let current_status = StatusModule::status_kind(boma);
        let is_furafura_stand = current_status == *FIGHTER_STATUS_KIND_FURAFURA_STAND; // 0x5F
        let is_furafura = current_status == *FIGHTER_STATUS_KIND_FURAFURA; // 0x60
        let is_bind = current_status == *FIGHTER_STATUS_KIND_BIND; // 0x66

        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        let instance_key = get_instance_key(boma) as usize;

        if entry_id < 8 {
            // Track per-status sound state
            static mut FURAFURA_SOUND_ACTIVE: [bool; 256] = [false; 256];
            static mut FURAFURA_START_FRAME: [i32; 256] = [-1; 256];
            static mut LAST_FURAFURA_STATUS: [i32; 256] = [-1; 256];
            static mut LAST_VANILLA_MUTE_FRAME: [i32; 256] = [-10; 256];
            
            let current_frame = player_state.current_frame;
            let status_changed = LAST_FURAFURA_STATUS[instance_key] != current_status;
            
            if is_bind {
                // BIND: Play for exactly 102 frames, then stop
                if !FURAFURA_SOUND_ACTIVE[instance_key] || status_changed {
                    // Stop any existing sound first
                    if FURAFURA_SOUND_ACTIVE[instance_key] {
                        SoundModule::stop_se(boma, Hash40::new("g_furafura"), 0);
                    }
                    
                    // Start new 102-frame duration sound
                    let sfx_handle = SoundModule::play_se(
                        boma,
                        Hash40::new("g_furafura"),
                        true, // Loop for 102 frames
                        false, false, false,
                        smash::app::enSEType(0)
                    ) as u32;
                    
                    SoundModule::set_se_vol(boma, sfx_handle as i32, 1.5, 0);
                    FURAFURA_SOUND_ACTIVE[instance_key] = true;
                    FURAFURA_START_FRAME[instance_key] = current_frame;
                    
                }
                
                // Check if 102 frames have elapsed for BIND
                if FURAFURA_SOUND_ACTIVE[instance_key] && 
                (current_frame - FURAFURA_START_FRAME[instance_key] >= 102) {
                    SoundModule::stop_se(boma, Hash40::new("g_furafura"), 0);
                    FURAFURA_SOUND_ACTIVE[instance_key] = false;
                }
                
            } else if is_furafura_stand || is_furafura {
                // FURAFURA/FURAFURA_STAND: One-shot play (will loop naturally)
                if !FURAFURA_SOUND_ACTIVE[instance_key] || status_changed {
                    // Stop any existing sound first
                    if FURAFURA_SOUND_ACTIVE[instance_key] {
                        SoundModule::stop_se(boma, Hash40::new("g_furafura"), 0);
                    }
                    
                    let sfx_handle = SoundModule::play_se(
                        boma,
                        Hash40::new("g_furafura"),
                        false, // One-shot for furafura statuses
                        false, false, false,
                        smash::app::enSEType(0)
                    ) as u32;
                    
                    SoundModule::set_se_vol(boma, sfx_handle as i32, 1.5, 0);
                    FURAFURA_SOUND_ACTIVE[instance_key] = true;
                    FURAFURA_START_FRAME[instance_key] = current_frame;
                    
                }
                
            } else {
                // Not in any furafura status - stop sound
                if FURAFURA_SOUND_ACTIVE[instance_key] {
                    SoundModule::stop_se(boma, Hash40::new("g_furafura"), 0);
                    FURAFURA_SOUND_ACTIVE[instance_key] = false;
                }
            }
            
            // VANILLA SOUND MUTING: Mute during any furafura status (every 10 frames to avoid spam)
            if (is_bind || is_furafura_stand || is_furafura) && 
            (current_frame - LAST_VANILLA_MUTE_FRAME[instance_key] >= 10) {
                SoundModule::stop_se(boma, Hash40::new("se_common_dizzy_add"), 0);
                SoundModule::stop_se(boma, Hash40::new("se_common_dizzy_loop"), 0);
                LAST_VANILLA_MUTE_FRAME[instance_key] = current_frame;
            }
            
            LAST_FURAFURA_STATUS[instance_key] = current_status;
        }
        
        // ===== G_GRAB_BURN SOUND (during catch statuses - ACMD handles stage filtering) =====
        let current_status = StatusModule::status_kind(boma);
        let is_catch_status = current_status == *FIGHTER_STATUS_KIND_CATCH_WAIT ||
                            current_status == *FIGHTER_STATUS_KIND_CATCH_PULL;

        if is_catch_status {
            if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE) {
                WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE);
                
                let sfx_handle = SoundModule::play_se(
                    boma,
                    Hash40::new("g_grab_burn"),
                    true,
                    false, false, false,
                    smash::app::enSEType(0)
                ) as u32;
                
                G_GRAB_BURN_HANDLE[instance_key] = sfx_handle;
                SoundModule::set_se_vol(boma, sfx_handle as i32, 1.8, 0);
            }
        } else {
            if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE) {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_GRAB_BURN_ACTIVE);
                
                if G_GRAB_BURN_HANDLE[instance_key] != 0 {
                    SoundModule::stop_se_handle(boma, G_GRAB_BURN_HANDLE[instance_key] as i32, 0);
                    G_GRAB_BURN_HANDLE[instance_key] = 0;
                }
                SoundModule::stop_se(boma, Hash40::new("g_grab_burn"), 0);
            }
        }
        // ===== MEGASYMBOL SOUND (84 frames during final smash - once per FS) =====
        let is_final_smash = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);

        static mut MEGASYMBOL_PLAYED_THIS_FS: [bool; 256] = [false; 256];
        static mut LAST_FS_FLAG: [bool; 256] = [false; 256];

        // Reset flag when final smash starts
        if is_final_smash && !LAST_FS_FLAG[instance_key] {
            MEGASYMBOL_PLAYED_THIS_FS[instance_key] = false;
        }

        // Reset flag when final smash ends
        if !is_final_smash && LAST_FS_FLAG[instance_key] {
            MEGASYMBOL_PLAYED_THIS_FS[instance_key] = false;
        }

        LAST_FS_FLAG[instance_key] = is_final_smash;

        if is_final_smash && !MEGASYMBOL_PLAYED_THIS_FS[instance_key] {
            if !WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE) {
                WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE);
                WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER);
                
                let sfx_handle = SoundModule::play_se(
                    boma,
                    Hash40::new("megasymbol"),
                    true,
                    false, false, false,
                    smash::app::enSEType(0)
                ) as u32;
                
                MEGASYMBOL_HANDLE[instance_key] = sfx_handle;
                SoundModule::set_se_vol(boma, sfx_handle as i32, 2.0, 0);
                MEGASYMBOL_PLAYED_THIS_FS[instance_key] = true;
            }
        }

// Always update timer if active
if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE) {
    let timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER);
    let new_timer = timer + 1.0;
    WorkModule::set_float(boma, new_timer, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_MEGASYMBOL_TIMER);
    
    // Stop after 84 frames
    if new_timer >= 84.0 {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE);
        
        if MEGASYMBOL_HANDLE[instance_key] != 0 {
            SoundModule::stop_se_handle(boma, MEGASYMBOL_HANDLE[instance_key] as i32, 0);
            MEGASYMBOL_HANDLE[instance_key] = 0;
        }
        SoundModule::stop_se(boma, Hash40::new("megasymbol"), 0);
        
    }
}

// Stop immediately if final smash ends
if !is_final_smash && WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE) {
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_MEGASYMBOL_ACTIVE);
    
    if MEGASYMBOL_HANDLE[instance_key] != 0 {
        SoundModule::stop_se_handle(boma, MEGASYMBOL_HANDLE[instance_key] as i32, 0);
        MEGASYMBOL_HANDLE[instance_key] = 0;
    }
    SoundModule::stop_se(boma, Hash40::new("megasymbol"), 0);
    
}
    
}

// Enhanced debug function with better formatting
unsafe fn debug_sound_state_enhanced(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let evolving_active = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVING_ACTIVE);
    let ss_active = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_EVOLVE_SS_ACTIVE);
    let charge_active = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_SHADOWBALL_CHARGE_ACTIVE);
    let evolving_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVING_TIMER);
    let ss_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_EVOLVE_SS_TIMER);
    
    let current_motion = MotionModule::motion_kind(boma);
    let is_jump_motion = current_motion == 0xe7f6e164a;
    
    if player_state.current_frame % 60 == 0 || is_jump_motion || (player_state.is_evolving && player_state.evolution_timer % 30 == 0) {
        
        if is_jump_motion {
        }
    }
}

unsafe fn reset_evolution_progress_on_match_start(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState
) {
    let instance_key = get_instance_key(boma);
    let current_status = StatusModule::status_kind(boma);
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let current_damage = DamageModule::damage(boma, 0);
    let current_frame = player_state.current_frame;

    // Enhanced reset detection for multiple scenarios
    static mut LAST_RESET_DAMAGE: [f32; 256] = [-1.0; 256];
    static mut LAST_RESET_FRAME: [i32; 256] = [-1; 256];

    let entry_idx = entry_id as usize;
    if entry_idx >= 8 { return; }

    // Debug reset detection
    if player_state.is_evolving {
    }

    let should_reset = 
        // Status-based reset - only very specific statuses
        current_status == *FIGHTER_STATUS_KIND_STANDBY ||
        current_status == *FIGHTER_STATUS_KIND_ENTRY ||
        current_status == *FIGHTER_STATUS_KIND_REBIRTH ||
        // ENHANCED: More aggressive training reset detection
        (current_status == 0x0 && current_frame < 100 && 
        ((instance_key as usize) < 256 && ((current_damage <= 0.1 && LAST_RESET_DAMAGE[instance_key as usize] > 10.0) ||
        (current_damage >= 30.0 && LAST_RESET_DAMAGE[instance_key as usize] <= 5.0)))) ||
        // Frame reset (new session) - tighter frame window
        ((instance_key as usize) < 256 && current_frame < 60 && LAST_RESET_FRAME[instance_key as usize] > 300) ||
        // ADDITIONAL: Direct damage reset to 0 from high values (training mode fix damage)
        ((instance_key as usize) < 256 && current_damage <= 0.1 && LAST_RESET_DAMAGE[instance_key as usize] > 20.0);

    if should_reset {

        // Cancel evolution if currently evolving
        if player_state.is_evolving {
            player_state.is_evolving = false;
            player_state.linking_cord_active = false;
            player_state.evolution_timer = 0;
            player_state.linking_cord_evo_attempt_icon_timer = 0;
            player_state.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
            
            // Apply evolution cancellation penalty
            player_state.evo_attempt_delay_damage_taken_penalty += 15.0;
            
        }

        // Reset ALL evolution progress
        player_state.damage_received_this_stage = 0.0;
        player_state.hits_landed_this_stage = 0;
        player_state.evo_attempt_delay_damage_taken_penalty = 0.0;
        player_state.evo_attempt_delay_hits_penalty = 0;
        player_state.previous_total_damage = current_damage;
        player_state.reset_evo_readiness_icons();
        player_state.last_evolution_confirmation_frame = -1;

        // Force hide all readiness icons
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
        ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);

    }

    // Update tracking
    if (instance_key as usize) < 256 {
        LAST_RESET_DAMAGE[instance_key as usize] = current_damage;
        LAST_RESET_FRAME[instance_key as usize] = current_frame;
    }
}

// Helper functions
unsafe fn handle_timed_looping_sound(
    boma: *mut BattleObjectModuleAccessor,
    sound_name: &str,
    max_frames: f32,
    flag_id: i32,
    timer_id: i32,
    volume: f32
) {
    if WorkModule::is_flag(boma, flag_id) {
        let timer = WorkModule::get_float(boma, timer_id);
        WorkModule::set_float(boma, timer + 1.0, timer_id);
        
        if timer >= max_frames {
            stop_looping_sound(boma, sound_name, flag_id);
        }
    } else {
        WorkModule::set_flag(boma, true, flag_id);
        WorkModule::set_float(boma, 0.0, timer_id);
        
        let sfx_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound_name),
            true, // Loop
            false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, sfx_handle as i32, volume, 0);
    }
}

unsafe fn handle_continuous_looping_sound(
    boma: *mut BattleObjectModuleAccessor,
    sound_name: &str,
    flag_id: i32,
    volume: f32
) {
    if !WorkModule::is_flag(boma, flag_id) {
        WorkModule::set_flag(boma, true, flag_id);
        
        let sfx_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound_name),
            true, // Loop
            false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, sfx_handle as i32, volume, 0);
    }
}

unsafe fn stop_looping_sound(
    boma: *mut BattleObjectModuleAccessor,
    sound_name: &str,
    flag_id: i32
) {
    if WorkModule::is_flag(boma, flag_id) {
        WorkModule::set_flag(boma, false, flag_id);
        SoundModule::stop_se(boma, Hash40::new(sound_name), 0);
    }
}

// Enhanced check_effect_exists function with better detection
unsafe fn check_effect_exists(boma: *mut BattleObjectModuleAccessor, effect_name: &str) -> bool {
    let current_status = StatusModule::status_kind(boma);
    
    match effect_name {
        "ridley_grabbing_catch" => {
            // Check if we're in grab statuses where this effect would exist
            let is_catch_status = current_status == *FIGHTER_STATUS_KIND_CATCH_WAIT ||
                                 current_status == *FIGHTER_STATUS_KIND_CATCH_PULL;
            
            // Additional check: Make sure we're actually Gastly stage for this effect
            let instance_key = get_instance_key(boma);
            let is_gastly_stage = {
                let states_map = crate::gastly::FIGHTER_STATES.read();
                states_map.get(&instance_key)
                    .map(|state| state.stage == crate::gastly::player_state::EvolutionStage::Gastly)
                    .unwrap_or(false)
            };
            
            is_catch_status && is_gastly_stage
        },
        
        "lucario_final_megasymbol" => {
            // Check if we're in final smash status and have the right conditions
            let is_final_smash = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
            let instance_key = get_instance_key(boma);
            
            // Check if this player has mega mode active
            let has_mega_mode = {
                let states_map = crate::gastly::FIGHTER_STATES.read();
                states_map.get(&instance_key)
                    .map(|state| state.mega_gengar_form_active && state.stage == crate::gastly::player_state::EvolutionStage::Gengar)
                    .unwrap_or(false)
            };
            
            is_final_smash && has_mega_mode
        },
        
        _ => false
    }
}

unsafe fn handle_damage_based_sounds(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState
) {
    let current_damage = DamageModule::damage(boma, 0);
    let previous_damage = player_state.previous_total_damage;
    
    // G_POTION: Heal of >= 15% (damage goes negative, meaning heal occurred)
    let damage_change = current_damage - previous_damage;
    if damage_change <= -15.0 { // Negative means healing, >= 15% heal
        handle_timed_looping_sound(
            boma, "g_potion", 50.0,
            FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE,
            FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER,
            1.0
        );
        
        // Stop g_restore if it's playing (as per your logic requirements)
        stop_looping_sound(boma, "g_restore", FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
        
    }
    
    // G_RESTORE: Heal to zero percent from having above 25% damage previously
    if current_damage <= 0.1 && previous_damage >= 25.0 { // Healed to ~0% from 25%+
        handle_timed_looping_sound(
            boma, "g_restore", 105.0,
            FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE,
            FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER,
            1.0
        );
        
        // Stop g_potion if it's playing (as per your logic requirements)
        stop_looping_sound(boma, "g_potion", FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
        
    }
    
    // Stop both sounds if no healing conditions are met
    if damage_change >= 0.0 { // No healing occurred
        // Check if timers have expired and stop sounds naturally
        let potion_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_POTION_TIMER);
        let restore_timer = WorkModule::get_float(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_G_RESTORE_TIMER);
        
        if potion_timer >= 50.0 {
            stop_looping_sound(boma, "g_potion", FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_POTION_ACTIVE);
        }
        
        if restore_timer >= 105.0 {
            stop_looping_sound(boma, "g_restore", FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_G_RESTORE_ACTIVE);
        }
    }
}

unsafe extern "C" fn gastly_early_frame_callback(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    if boma.is_null() { return; }

    let fighter_kind_val: i32 = utility::get_kind(&mut *boma);
    if fighter_kind_val != *FIGHTER_KIND_PURIN { return; }

    let my_entry_id_i32 = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let my_entry_id_u32 = my_entry_id_i32 as u32;
    let instance_key = get_instance_key(boma);
    let current_status = StatusModule::status_kind(boma);

    // Early marked costume reset detection
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    let is_marked_costume = if color_id < 256 {
        unsafe { crate::MARKED_COLORS[color_id] }
    } else {
        false
    };

    if is_marked_costume {
    static mut LAST_SEEN_FRAME: [i32; 256] = [-1; 256];
    let entry_idx = my_entry_id_u32 as usize;
    
    if entry_idx < 8 {
        let current_status = StatusModule::status_kind(boma);
        let current_damage = DamageModule::damage(boma, 0);
        
        let current_frame = {
            let states_map_reader = FIGHTER_STATES.read();
            states_map_reader.get(&instance_key)
                .map(|state| state.current_frame)
                .unwrap_or(0)
        };
        
        // Detect if this is a fresh session by checking for frame reset
        let frame_jumped_backwards = (instance_key as usize) < 256 && LAST_SEEN_FRAME[instance_key as usize] != -1 && 
                                    current_frame < LAST_SEEN_FRAME[instance_key as usize] - 100;
        
        // Also check for early frame + entry status
        let is_early_entry = current_frame < 30 && 
                            (current_status == *FIGHTER_STATUS_KIND_ENTRY || 
                             current_status == *FIGHTER_STATUS_KIND_STANDBY);
        
        if frame_jumped_backwards || is_early_entry {
            let mut states_map_writer = FIGHTER_STATES.write();
            if let Some(player_state) = states_map_writer.get_mut(&instance_key) {
                if player_state.stage != crate::gastly::player_state::EvolutionStage::Gastly {
                    
                    // Force complete reset
                    player_state.full_reset_on_respawn(boma);
                    player_state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.evolution_target_stage = crate::gastly::player_state::EvolutionStage::Gastly;
                    player_state.is_shiny = detect_shiny_slot(boma);
                    
                    // Force visual reset
                    crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, crate::gastly::player_state::EvolutionStage::Gastly);
                    crate::gastly::visuals::set_active_eye_mesh(boma, player_state, None);
                    
                    // Reset UI state
                    crate::gastly::ui_management::reset_ui_state_on_death(my_entry_id_u32);
                }
            }
        }
        
        if (instance_key as usize) < 256 {
            LAST_SEEN_FRAME[instance_key as usize] = current_frame;
        }
    }
}

    // First-frame detection for marked costumes
    static mut FIRST_FRAME_PROCESSED: [bool; 256] = [false; 256];
    let current_status_val = StatusModule::status_kind(boma);
    let instance_key = get_instance_key(boma);
    let instance_idx_frame = instance_key as usize;

    if instance_idx_frame < 256 && !FIRST_FRAME_PROCESSED[instance_idx_frame] {
        FIRST_FRAME_PROCESSED[instance_idx_frame] = true;
        
        let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        if color_id < 256 && unsafe { crate::MARKED_COLORS[color_id] } {
            // Force complete state reset for marked costumes
            let instance_key = get_instance_key(boma);
            let mut states_map_reset = FIGHTER_STATES.write();
            if let Some(state) = states_map_reset.get_mut(&instance_key) {
                state.full_reset_on_respawn(boma);
                state.stage = crate::gastly::player_state::EvolutionStage::Gastly;
            }
        }
    }

    // Reset first frame flag on death/standby
    let current_status_val = StatusModule::status_kind(boma);
    if current_status_val == *FIGHTER_STATUS_KIND_DEAD || 
    current_status_val == *FIGHTER_STATUS_KIND_STANDBY {
        if instance_idx_frame < 256 {
            FIRST_FRAME_PROCESSED[instance_idx_frame] = false;
        }
    }

    let my_entry_id_i32 = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID);
    let instance_key = get_instance_key(boma);
    let current_frame = {
        let states_map_reader = FIGHTER_STATES.read();
        states_map_reader.get(&instance_key)
            .map(|state| state.current_frame)
            .unwrap_or(0)
    };
}

pub fn install() {
    skyline::install_hooks!(hit_tracking_hook);

    crate::gastly::agent_init::install();
    crate::gastly::animation_hooks::install_animation_hooks();
    crate::gastly::effects::install_effects();
    crate::gastly::darkfx::install_dark_effects();
}

// New function to install frame callbacks with costume filtering
pub fn install_frame_callbacks_with_costumes() {
    // Create costume vector from marked colors
    let mut costume = Vec::new();
    unsafe {
        for i in 0..256 {
            if crate::MARKED_COLORS[i] {
                costume.push(i);
            }
        }
    }
    
    if costume.is_empty() {
        return;
    }
    
    
    smashline::Agent::new("purin")
        .set_costume(costume.clone())
        .on_line(smashline::Main, gastly_early_frame_callback)
        .on_line(smashline::Main, gastly_fighter_frame_callback)
        .on_start(init_gastly_aura)
        .install();

    smashline::Agent::new("fighter")
        .set_costume(costume)
        .on_line(smashline::Main, gastly_global_fighter_frame)
        .install();
}

// New function to install ACMD with costume filtering
pub fn install_acmd_with_costumes() {
    // Create costume vector from marked colors
    let mut costume = Vec::new();
    unsafe {
        for i in 0..256 {
            if crate::MARKED_COLORS[i] {
                costume.push(i);
            }
        }
    }
    
    if costume.is_empty() {
        return;
    }
    
    
    // Install with costume filtering
    crate::gastly::sounds::install_sound_logic_with_costumes(&costume);
    crate::gastly::acmdsound::install_acmd_sound_with_costumes(&costume);
    crate::gastly::attack_voices::install_attack_voices_remaining_with_costumes(&costume);
    crate::gastly::acmd::install_acmd_with_costumes(&costume);
}