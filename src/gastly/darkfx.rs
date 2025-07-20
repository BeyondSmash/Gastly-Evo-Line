// src/gastly/darkfx.rs

use smash::app::lua_bind::{
    StatusModule, MotionModule, WorkModule, ModelModule, EffectModule, 
    AttackModule, DamageModule, StopModule, SoundModule
};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::{Hash40, Vector3f};
use smash::lib::lua_const::*;
use smash::lua2cpp::L2CFighterCommon;
use smash::app::utility;
use smash_script::macros;
use smashline::{Agent, Main};

use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// Import for evolution stage detection
use crate::gastly::{FIGHTER_STATES, player_state::EvolutionStage};

// --- DARK EFFECT CONFIGURATION ---
const DARK_EFFECT_NAME: &str = "ganon_attack_purple";
const DARK_EFFECT_COOLDOWN_FRAMES: u32 = 30;

// Dark move type constants - UPDATED FOR NEW EFFECTS
const DARK_MOVE_DOWN_TILT: u32 = 2;
const DARK_MOVE_DOWN_SMASH: u32 = 7;
const DARK_MOVE_NEUTRAL_AIR: u32 = 8;
const DARK_MOVE_FORWARD_AIR: u32 = 9;
const DARK_MOVE_DOWN_AIR: u32 = 10; // NEW: Down Air with bomber sweat (Gastly/Haunter only)
const DARK_MOVE_DOWN_SPECIAL: u32 = 12; // Down Special with poison effect
const DARK_MOVE_NEUTRAL_SPECIAL_ROLLOUT: u32 = 13; // For rollout flash effect
const DARK_MOVE_FORWARD_SPECIAL: u32 = 14;
const DARK_MOVE_PUMMEL: u32 = 16; // NEW: Pummel with bomber sweat (All stages)

// Global frame counter
static mut PSEUDO_GLOBAL_FRAME_COUNT: u32 = 0;

#[derive(Clone)]
struct AttackerInfo {
    entry_id: u32,
    move_type: u32,
    evolution_stage: u32, // 0=Gastly, 1=Haunter, 2=Gengar
}

#[derive(Debug, Clone, Copy)]
struct DarkEffectData {
    last_effect_frame: u32,
    flash_stage: u8,
    flash_timer: i32,
    // Enhanced flash tracking for status interruption handling
    flash_total_planned_duration: i32,
    flash_elapsed_duration: i32,
    flash_last_status: i32,
}

impl Default for DarkEffectData {
    fn default() -> Self {
        Self {
            last_effect_frame: 0,
            flash_stage: 0,
            flash_timer: 0,
            flash_total_planned_duration: 0,
            flash_elapsed_duration: 0,
            flash_last_status: -1,
        }
    }
}

static DARK_EFFECT_DATA: Lazy<Mutex<HashMap<u32, DarkEffectData>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// Store which fighters are Gastly performers of dark moves this frame with move info
static GASTLY_ATTACKERS_THIS_FRAME: Lazy<Mutex<Vec<AttackerInfo>>> = 
    Lazy::new(|| Mutex::new(Vec::new()));

// Get evolution stage from player state
unsafe fn get_evolution_stage(boma: *mut BattleObjectModuleAccessor) -> u32 {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    // Access FIGHTER_STATES to get the actual evolution stage
    let states_map = crate::gastly::FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&entry_id) {
        return match player_state.stage {
            crate::gastly::player_state::EvolutionStage::Gastly => 0,
            crate::gastly::player_state::EvolutionStage::Haunter => 1, 
            crate::gastly::player_state::EvolutionStage::Gengar => 2,
        };
    }
    
    // Default to Gastly if unable to read state
    0
}

unsafe fn check_for_dark_move(boma: *mut BattleObjectModuleAccessor) -> Option<(u32, u32)> {
    // FIRST: Must be a Gastly (Purin)
    let fighter_kind = utility::get_kind(&mut *boma);
    if fighter_kind != *FIGHTER_KIND_PURIN {
        return None;
    }

    // Check if this costume slot is marked for Gastly mod
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    if color_id >= 256 || !crate::MARKED_COLORS[color_id] {
        return None;
    }

    let evolution_stage = get_evolution_stage(boma);
    let current_status = StatusModule::status_kind(boma);
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    
    let move_type = match current_status {
        // ONLY THESE MOVES CAUSE DARK EFFECTS:
        s if s == *FIGHTER_STATUS_KIND_ATTACK_LW3 => Some(DARK_MOVE_DOWN_TILT), // Down Tilt
        s if s == *FIGHTER_STATUS_KIND_ATTACK_LW4 => Some(DARK_MOVE_DOWN_SMASH), // Down Smash
        s if s == *FIGHTER_STATUS_KIND_ATTACK_AIR => {
            if motion_hash.hash == smash::hash40("attack_air_n") {
                Some(DARK_MOVE_NEUTRAL_AIR) // Neutral Air
            } else if motion_hash.hash == smash::hash40("attack_air_f") {
                Some(DARK_MOVE_FORWARD_AIR) // Forward Air
            } else if motion_hash.hash == smash::hash40("attack_air_lw") {
                // NEW: Down Air with bomber sweat (Gastly/Haunter only)
                if evolution_stage == 0 || evolution_stage == 1 { // Gastly or Haunter
                    Some(DARK_MOVE_DOWN_AIR)
                } else {
                    None // Gengar omitted from down air effect
                }
            } else {
                None
            }
        },
        s if s == *FIGHTER_STATUS_KIND_SPECIAL_S => Some(DARK_MOVE_FORWARD_SPECIAL), // Forward Special
        s if s == *FIGHTER_STATUS_KIND_SPECIAL_LW => Some(DARK_MOVE_DOWN_SPECIAL), // Down Special
        // FIXED: Neutral Special hit status (when shadowball bomb occurs)
        s if s == 0x1E7 => { // SPECIAL_N_HIT_END status
            Some(DARK_MOVE_NEUTRAL_SPECIAL_ROLLOUT)
        },
        s if s == *FIGHTER_STATUS_KIND_CATCH_ATTACK => Some(DARK_MOVE_PUMMEL), // NEW: Pummel with bomber sweat (All stages)
        _ => None,
    };
    
    move_type.map(|mt| (mt, evolution_stage))
}

// Begin enhanced purple flash with status change detection
unsafe fn begin_enhanced_purple_flash(entry_id: u32, total_duration: i32) {
    if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
        let data = data_map.entry(entry_id).or_insert_with(DarkEffectData::default);
        data.flash_stage = 1;
        data.flash_timer = 8; // Start with first blink
        data.flash_total_planned_duration = total_duration;
        data.flash_elapsed_duration = 0;
        data.flash_last_status = -1; // Will be set on first update
    }
}

// Enhanced flash effects with status change detection
unsafe fn update_purple_flash_effects_with_status_detection(fighter: &mut L2CFighterCommon, entry_id: u32, current_status: i32) {
    if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
        if let Some(data) = data_map.get_mut(&entry_id) {
            // Check for status change during enhanced flash
            if data.flash_total_planned_duration > 0 {
                if data.flash_last_status == -1 {
                    data.flash_last_status = current_status; // Initialize
                } else if data.flash_last_status != current_status {
                    // Status changed! Restart flash if we haven't completed the planned duration
                    if data.flash_elapsed_duration < data.flash_total_planned_duration {
                        println!("[DARK EFFECTS] Status change detected during flash ({}->{}), restarting flash", 
                                data.flash_last_status, current_status);
                        data.flash_stage = 1;
                        data.flash_timer = 8;
                        data.flash_last_status = current_status;
                        // Don't reset elapsed_duration, keep accumulating
                    }
                }
                
                // Update elapsed duration
                data.flash_elapsed_duration += 1;
                
                // Check if we've completed the total planned duration
                if data.flash_elapsed_duration >= data.flash_total_planned_duration {
                    data.flash_stage = 0;
                    data.flash_total_planned_duration = 0;
                    data.flash_elapsed_duration = 0;
                    macros::COL_NORMAL(fighter);
                    return;
                }
            }
            
            // Regular flash processing (enhanced version)
            if data.flash_stage > 0 {
                data.flash_timer -= 1;
                
                match data.flash_stage {
                    1 => { // First blink
                        macros::FLASH(fighter, 0.2, 0.05, 0.3, 0.8); // Darker purple for regular dark moves
                        if data.flash_timer <= 0 {
                            data.flash_stage = 2;
                            data.flash_timer = 6;
                        }
                    },
                    2 => { // Gap between first and second blink
                        macros::COL_NORMAL(fighter);
                        if data.flash_timer <= 0 {
                            data.flash_stage = 3;
                            data.flash_timer = 8;
                        }
                    },
                    3 => { // Second blink
                        macros::FLASH(fighter, 0.2, 0.05, 0.3, 0.8); // Darker purple for regular dark moves
                        if data.flash_timer <= 0 {
                            data.flash_stage = 4;
                            data.flash_timer = 6;
                        }
                    },
                    4 => { // Gap between second and third blink
                        macros::COL_NORMAL(fighter);
                        if data.flash_timer <= 0 {
                            data.flash_stage = 5;
                            data.flash_timer = 8;
                        }
                    },
                    5 => { // Third blink
                        macros::FLASH(fighter, 0.2, 0.05, 0.3, 0.8); // Darker purple for regular dark moves
                        if data.flash_timer <= 0 {
                            data.flash_stage = 6;
                            data.flash_timer = 30; // 30 frame fade
                        }
                    },
                    6 => { // Fade out over 30 frames
                        let fade_progress = 1.0 - (data.flash_timer as f32 / 30.0);
                        let fade_alpha = 0.8 * (1.0 - fade_progress);
                        
                        if fade_alpha > 0.0 {
                            macros::FLASH(fighter, 0.2, 0.05, 0.3, fade_alpha); // Darker purple fade
                        } else {
                            macros::COL_NORMAL(fighter);
                        }
                        
                        if data.flash_timer <= 0 {
                            // Only end if we've completed the planned duration
                            if data.flash_total_planned_duration == 0 || 
                               data.flash_elapsed_duration >= data.flash_total_planned_duration {
                                data.flash_stage = 0;
                                data.flash_total_planned_duration = 0;
                                data.flash_elapsed_duration = 0;
                                macros::COL_NORMAL(fighter);
                            } else {
                                // Restart the sequence if we haven't hit the planned duration
                                data.flash_stage = 1;
                                data.flash_timer = 8;
                            }
                        }
                    },
                    _ => {
                        data.flash_stage = 0;
                        data.flash_total_planned_duration = 0;
                        data.flash_elapsed_duration = 0;
                        macros::COL_NORMAL(fighter);
                    }
                }
            }
        }
    }
}

// Start flash effect for attacker (simple version - darker purple)
unsafe fn begin_purple_flash(entry_id: u32) {
    if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
        let data = data_map.entry(entry_id).or_insert_with(DarkEffectData::default);
        if data.flash_stage == 0 {
            data.flash_stage = 1;
            data.flash_timer = 8;
            // Note: This uses the darker purple flash (0.2, 0.05, 0.3)
        }
    }
}

// Clean up flash effects on death/respawn
unsafe fn cleanup_flash_effects_on_death(entry_id: u32) {
    if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
        if let Some(data) = data_map.get_mut(&entry_id) {
            data.flash_stage = 0;
            data.flash_timer = 0;
            data.flash_total_planned_duration = 0;
            data.flash_elapsed_duration = 0;
            data.flash_last_status = -1;
        }
    }
}

// NEW: Main dark effects handler called from mod.rs
unsafe extern "C" fn gastly_dark_effects_handler(fighter: &mut L2CFighterCommon) {
    let module_accessor: *mut BattleObjectModuleAccessor = fighter.module_accessor;
    let fighter_entry_id_u32 = WorkModule::get_int(module_accessor, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;

    // Check for death/respawn status to clean up flash effects
    let current_status = StatusModule::status_kind(module_accessor);
    if current_status == *FIGHTER_STATUS_KIND_DEAD || current_status == *FIGHTER_STATUS_KIND_REBIRTH {
        cleanup_flash_effects_on_death(fighter_entry_id_u32);
        macros::COL_NORMAL(fighter); // Ensure normal color on death
        return;
    }

    // Update global frame counter (only for player 0)
    if fighter_entry_id_u32 == 0 { 
        PSEUDO_GLOBAL_FRAME_COUNT = PSEUDO_GLOBAL_FRAME_COUNT.wrapping_add(1);
        
        // Clear the list at the start of each frame
        if let Ok(mut attackers) = GASTLY_ATTACKERS_THIS_FRAME.lock() {
            attackers.clear();
        }
    }

    // PHASE 1: Collect Gastly attackers
    if let Some((move_type, evolution_stage)) = check_for_dark_move(module_accessor) {
        if move_type == DARK_MOVE_NEUTRAL_SPECIAL_ROLLOUT {
            // For 0x1E7 status, we don't need to check AttackModule - just add to attackers
            if let Ok(mut attackers) = GASTLY_ATTACKERS_THIS_FRAME.lock() {
                let attacker_info = AttackerInfo {
                    entry_id: fighter_entry_id_u32,
                    move_type,
                    evolution_stage,
                };
                
                if !attackers.iter().any(|a| a.entry_id == fighter_entry_id_u32) {
                    attackers.push(attacker_info);
                    println!("[DARK EFFECTS] Added neutral special hit end attacker {}", fighter_entry_id_u32);
                }
            }
        } else {
            // For all other moves, check for hits using AttackModule
            let infliction_hit = AttackModule::is_infliction_status(module_accessor, 0);
            let attack_occur = AttackModule::is_attack_occur(module_accessor);
            
            if infliction_hit || attack_occur {
                // This Gastly is hitting someone with a dark move
                if let Ok(mut attackers) = GASTLY_ATTACKERS_THIS_FRAME.lock() {
                    let attacker_info = AttackerInfo {
                        entry_id: fighter_entry_id_u32,
                        move_type,
                        evolution_stage,
                    };
                    
                    // Check if this attacker is already in the list
                    if !attackers.iter().any(|a| a.entry_id == fighter_entry_id_u32) {
                        attackers.push(attacker_info);
                        println!("[DARK EFFECTS] Added {} attacker {} (stage {})", 
                                match move_type {
                                    DARK_MOVE_DOWN_AIR => "down air",
                                    DARK_MOVE_PUMMEL => "pummel", 
                                    _ => "other"
                                }, fighter_entry_id_u32, evolution_stage);
                    }
                }
            }
        }
    }

    // PHASE 2: Apply effects to victims
    let current_pseudo_global_frame = PSEUDO_GLOBAL_FRAME_COUNT;

    // Check if this fighter is taking damage
    if StopModule::is_damage(module_accessor) {
        // Check if any Gastly is attacking this frame
        let gastly_attackers = {
            if let Ok(attackers) = GASTLY_ATTACKERS_THIS_FRAME.lock() {
                attackers.clone()
            } else {
                Vec::new()
            }
        };

        if !gastly_attackers.is_empty() {
            // This fighter is taking damage while Gastly is attacking - apply appropriate effect
            let mut should_spawn_effect = false;
            if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
                let data = data_map.entry(fighter_entry_id_u32).or_insert_with(DarkEffectData::default);
                
                // Check cooldown
                if current_pseudo_global_frame >= data.last_effect_frame.wrapping_add(DARK_EFFECT_COOLDOWN_FRAMES) {
                    should_spawn_effect = true;
                    data.last_effect_frame = current_pseudo_global_frame;
                }
            }

            if should_spawn_effect {
                // Get character model scale for proper effect sizing
                let character_model_scale = ModelModule::scale(module_accessor);
                let final_effect_scale = 1.75 * character_model_scale.max(0.1);
                
                // Determine what effects to apply based on the move types
                let mut has_down_special = false;
                let mut has_pummel = false;
                let mut has_down_air = false; // NEW
                let mut has_neutral_special_rollout = false;
                let mut has_regular_dark_move = false;
                
                for attacker in &gastly_attackers {
                    match attacker.move_type {
                        DARK_MOVE_DOWN_SPECIAL => has_down_special = true,
                        DARK_MOVE_PUMMEL => has_pummel = true,
                        DARK_MOVE_DOWN_AIR => has_down_air = true, // NEW
                        DARK_MOVE_NEUTRAL_SPECIAL_ROLLOUT => has_neutral_special_rollout = true,
                        // All actual dark moves (ganon effect + flash)
                        DARK_MOVE_DOWN_TILT | DARK_MOVE_DOWN_SMASH | 
                        DARK_MOVE_NEUTRAL_AIR | DARK_MOVE_FORWARD_AIR | 
                        DARK_MOVE_FORWARD_SPECIAL => has_regular_dark_move = true,
                        _ => {} // No other moves should reach here
                    }
                }
                
                // Apply effects based on move priority: Down Special > Down Air > Neutral Special Rollout > Pummel > Regular Dark Moves
                if has_down_special {
                    // Down Special - poison effect + enhanced flash
                    let effect_handle = EffectModule::req_follow(
                        module_accessor,
                        Hash40::new("sys_hit_poison"),
                        Hash40::new("hip"),
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        final_effect_scale,
                        true, 0x40000, 0, -1, 0, 0, false, false
                    );
                    
                    if effect_handle != u64::MAX && effect_handle != 0u64 {
                        let handle_u32 = effect_handle as u32;
                        EffectModule::set_rgb(module_accessor, handle_u32, 2.0, 1.0, 1.0);
                        EffectModule::set_rate(module_accessor, handle_u32, 0.25);
                    }
                    
                    // Start enhanced flash (120 frames = 3 blinks + fade)
                    begin_enhanced_purple_flash(fighter_entry_id_u32, 120);
                    
                } else if has_down_air {
                    // NEW: Down Air - sys_bomber_sweat effect (NO FLASH) - Gastly/Haunter only
                    let effect_handle = EffectModule::req_follow(
                        module_accessor,
                        Hash40::new("sys_bomber_sweat"),
                        Hash40::new("hip"),
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        &Vector3f { x: 0.0, y: 90.0, z: 0.0 }, // rot.xyz: 0, 90, 0 (emit upward)
                        1.0, // Fixed scale - will be adjusted with set_scale
                        true, 0x40000, 0, -1, 0, 0, false, false
                    );
                    
                    if effect_handle != u64::MAX && effect_handle != 0u64 {
                        let handle_u32 = effect_handle as u32;
                        // Set scale.xyz to 1, 2, 1
                        let scale_vector = Vector3f { x: 1.0, y: 2.0, z: 1.0 };
                        EffectModule::set_scale(module_accessor, handle_u32, &scale_vector);
                        // Reduce lifetime to make it despawn sooner
                        EffectModule::set_rate(module_accessor, handle_u32, 1.5); // 1.5x speed = shorter duration
                    }
                    
                    // NO FLASH for down air
                    println!("[DARK EFFECTS] sys_bomber_sweat (down air) effect on victim {}", fighter_entry_id_u32);
                    
                } else if has_neutral_special_rollout {
                    // Neutral Special Rollout - ganon effect + purple flash
                    let effect_handle = EffectModule::req_follow(
                        module_accessor,
                        Hash40::new("ganon_attack_purple"),
                        Hash40::new("hip"),
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        final_effect_scale,
                        true, 0x40000, 0, -1, 0, 0, false, false
                    );
                    
                    if effect_handle != u64::MAX && effect_handle != 0u64 {
                        let handle_u32 = effect_handle as u32;
                        EffectModule::set_rgb(module_accessor, handle_u32, 2.0, 1.0, 1.0);
                        EffectModule::set_rate(module_accessor, handle_u32, 0.3);
                    }
                    
                    begin_purple_flash(fighter_entry_id_u32);
                    println!("[DARK EFFECTS] Ganon effect + purple flash (rollout) on victim {}", fighter_entry_id_u32);
                    
                } else if has_pummel {
                    // NEW: Pummel (All stages): sys_bomber_sweat effect (NO FLASH)
                    let effect_handle = EffectModule::req_follow(
                        module_accessor,
                        Hash40::new("sys_bomber_sweat"),
                        Hash40::new("hip"),
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        &Vector3f { x: 0.0, y: 90.0, z: 0.0 }, // rot.xyz: 0, 90, 0 (emit upward)
                        1.0, // Fixed scale - will be adjusted with set_scale
                        true, 0x40000, 0, -1, 0, 0, false, false
                    );
                    
                    if effect_handle != u64::MAX && effect_handle != 0u64 {
                        let handle_u32 = effect_handle as u32;
                        // Set scale.xyz to 1, 2, 1
                        let scale_vector = Vector3f { x: 1.0, y: 2.0, z: 1.0 };
                        EffectModule::set_scale(module_accessor, handle_u32, &scale_vector);
                        // Reduce lifetime to make it despawn sooner
                        EffectModule::set_rate(module_accessor, handle_u32, 1.5); // 1.5x speed = shorter duration
                    }
                    
                    // NO FLASH for pummel effects
                    println!("[DARK EFFECTS] sys_bomber_sweat (pummel) effect on victim {}", fighter_entry_id_u32);
                    
                } else if has_regular_dark_move {
                    // Regular dark moves: ganon_attack_purple effect + purple flash
                    let effect_handle = EffectModule::req_follow(
                        module_accessor,
                        Hash40::new(DARK_EFFECT_NAME),
                        Hash40::new("hip"),
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                        final_effect_scale,
                        true, 0x40000, 0, -1, 0, 0, false, false
                    );
                    
                    if effect_handle != u64::MAX && effect_handle != 0u64 {
                        let handle_u32 = effect_handle as u32;
                        EffectModule::set_rgb(module_accessor, handle_u32, 2.0, 1.0, 1.0);
                        EffectModule::set_rate(module_accessor, handle_u32, 0.3);
                    }
                    
                    begin_purple_flash(fighter_entry_id_u32);
                }
            }
        }
    }

    // PHASE 3: Update flash effects for this fighter
    update_purple_flash_effects_with_status_detection(fighter, fighter_entry_id_u32, current_status);
}

// Update existing interface functions
pub unsafe fn process_dark_move_effects(
    boma: *mut BattleObjectModuleAccessor,
    fighter: &mut L2CFighterCommon,
    _current_frame: i32
) {
    // Handle flash effects for ALL fighters (both Gastly attackers AND victims)
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let current_status = StatusModule::status_kind(boma);
    update_purple_flash_effects_with_status_detection(fighter, entry_id, current_status);
}

pub fn install_dark_effects() {
    // Install the dark effects handler on all fighters
    smashline::Agent::new("fighter")
        .on_line(smashline::Main, gastly_dark_effects_handler)
        .install();
    
    println!("[DARK EFFECTS] Dark effects system installed with proper hooks!");
}

// Public function to clean up flash effects (call from your main mod on death/respawn)
pub unsafe fn cleanup_dark_effects_on_death(entry_id: u32) {
    cleanup_flash_effects_on_death(entry_id);
    
    // Additional cleanup for training mode resets
    if let Ok(mut data_map) = DARK_EFFECT_DATA.lock() {
        if let Some(data) = data_map.get_mut(&entry_id) {
            // Force reset ALL flash state
            data.flash_stage = 0;
            data.flash_timer = 0;
            data.flash_total_planned_duration = 0;
            data.flash_elapsed_duration = 0;
            data.flash_last_status = -1;
            
            println!("[DARK_CLEANUP] Force reset all flash state for entry {}", entry_id);
        }
    }
}

// NEW: Enhanced function to get a fighter's L2CFighterCommon and force COL_NORMAL
pub unsafe fn force_clear_flash_for_entry(entry_id: u32) {
    // Clean up the data state
    cleanup_dark_effects_on_death(entry_id);
    
    // Try to get the fighter's module accessor and force COL_NORMAL
    let fighter_boma = smash::app::sv_battle_object::module_accessor(entry_id);
    if !fighter_boma.is_null() {
        // Force reset color blend
        smash::app::lua_bind::ColorBlendModule::cancel_main_color(fighter_boma, 0);
        println!("[DARK_CLEANUP] Forced ColorBlend reset for entry {}", entry_id);
    }
}

// NEW: Global cleanup function that can be called from agent_init
pub unsafe fn force_cleanup_all_flash_effects() {
    // Clean up all players
    for entry_id in 0..8u32 {
        force_clear_flash_for_entry(entry_id);
    }
    println!("[DARK_CLEANUP] Force cleaned all flash effects for all players");
} 