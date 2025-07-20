use smash::app::lua_bind::{ModelModule, WorkModule, MotionModule, StatusModule, AttackModule, EffectModule, PostureModule, DamageModule};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::{Hash40, Vector3f};
use smash::lib::lua_const::*;
use smash::app::utility;

// Import constants (mesh names, motion hashes)
use crate::gastly::constants::*;
// Import PlayerEvolutionState and related enums
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage, BlinkPhase};

// COMPLETELY ISOLATED weakened state system
static mut WEAKENED_SPAWNED: [bool; 8] = [false; 8];
static mut WEAKENED_LAST_DAMAGE: [f32; 8] = [0.0; 8];
static mut DEBUG_LAST_FRAME_CHECKED: [i32; 8] = [-1; 8];

// Enhanced shadowball state enum with charge tracking
#[derive(Debug, PartialEq)]
pub enum ShadowballState {
    NotActive,
    ChargingBelowThreshold,
    ActiveFrameBased,
    ActiveWithHitbox,
    RegularRollout,
    ChargedRollout,
    AirToGroundRollout, // NEW: Ground roll after insufficient air charge
    TransitionKeepModel,
    RegularRolloutWithHitbox,      // Visible rollout but hitbox active
    ChargedRolloutWithHitbox,      // Invisible rollout with hitbox active  
    AirToGroundRolloutWithHitbox,  // Air-to-ground rollout with hitbox
}

#[derive(Debug, Copy, Clone)]
struct SimpleHitState {
    was_hitting: bool,
    last_bomb_frame: i32,
}

impl SimpleHitState {
    const fn new() -> Self {
        Self {
            was_hitting: false,
            last_bomb_frame: -100,
        }
    }
}

// NEW: Centralized function to check if Gastly body should be hidden
unsafe fn should_force_hide_gastly_body(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) -> bool {
    // Only hide for Haunter and Gengar stages during final smash
    if player_state.stage != EvolutionStage::Haunter && player_state.stage != EvolutionStage::Gengar {
        return false;
    }
    
    // Check multiple conditions to be absolutely sure
    let is_final_smash_flag = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
    let is_final_smash_form = player_state.is_in_final_smash_form;
    
    // Also check for final smash animations as backup
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    let is_final_animation = motion_hash.hash == smash::hash40("final") ||
                            motion_hash.hash == smash::hash40("final_air") ||
                            motion_hash.hash == smash::hash40("final_start_r") ||
                            motion_hash.hash == smash::hash40("final_end_r");
    
    is_final_smash_flag || is_final_smash_form || is_final_animation
}

// NEW: Force hide Gastly body if conditions are met (call this after any mesh visibility setting)
unsafe fn enforce_gastly_body_hiding(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) {
    if should_force_hide_gastly_body(boma, player_state) {
        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
    }
}

// NEW: Check for animations that should show gengar_tongue_normal for Gengar (non-FS modes)
unsafe fn should_show_gengar_tongue_normal(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) -> bool {
    // Only for Gengar stage and not in final smash forms
    if player_state.stage != EvolutionStage::Gengar || 
       player_state.giga_gengar_form_active || 
       player_state.mega_gengar_form_active {
        return false;
    }
    
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    
    // List of animations that should show gengar_tongue_normal
    let tongue_normal_animations = [
        smash::hash40("final"),
        smash::hash40("final_air"),
        smash::hash40("special_hi"),          
        smash::hash40("special_air_hi"),     
        smash::hash40("special_air_s"),
        smash::hash40("attack_dash"),
        smash::hash40("attack_s3_hi"),     
        smash::hash40("attack_s3_lw"),    
        smash::hash40("attack_s3_s"),    
        smash::hash40("attack_s4_s"),
        smash::hash40("attack_hi4"),
        smash::hash40("attack_lw4"),
        smash::hash40("attack_air_hi"),    
        smash::hash40("attack_air_f"),      
        smash::hash40("attack_air_n"),        
        smash::hash40("dash"),
        smash::hash40("turn_dash"),
        smash::hash40("attack_hi3"),
        smash::hash40("attack_lw3"),
        smash::hash40("attack_lw4_hold"),
        smash::hash40("special_s"),
        smash::hash40("damage_hi_1"),
        smash::hash40("damage_hi_2"),
        smash::hash40("damage_hi_3"),
        smash::hash40("damage_lw_1"),
        smash::hash40("damage_lw_2"),
        smash::hash40("damage_lw_3"),
        smash::hash40("damage_n_1"),
        smash::hash40("damage_n_2"),
        smash::hash40("damage_n_3"),
        smash::hash40("damage_fly_hi"),
        smash::hash40("damage_fly_lw"),
        smash::hash40("damage_fly_n"),
    ];
    
    tongue_normal_animations.contains(&motion_hash.hash)
}

// NEW: Check for attack_air_lw animation with frame-specific tongue visibility
unsafe fn should_show_tongue_for_attack_air_lw(boma: *mut BattleObjectModuleAccessor, _player_state: &PlayerEvolutionState) -> bool {
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    
    if motion_hash.hash == smash::hash40("attack_air_lw") {
        let motion_frame = MotionModule::frame(boma);
        
        // Show tongue during frames 7-49
        if motion_frame >= 7.0 && motion_frame <= 49.0 {
            return true;
        }
    }
    
    false
}

unsafe fn hide_all_evolving_meshes(boma: *mut BattleObjectModuleAccessor) {
    // Hide all evolving meshes
    ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING_FLOORSHADOW, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING_SHADOWBALL, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING_TONGUE, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_EVOLVING, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_EVOLVING_FLOORSHADOW, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_EVOLVING_SHADOWBALL, false);
}

// Gastly Evolving Tongue Swap

unsafe fn handle_evolving_tongue_visibility(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState
) -> bool {
    // Only during Gastly → Haunter evolution
    if !player_state.is_evolving || 
       player_state.stage != EvolutionStage::Gastly || 
       player_state.evolution_target_stage != EvolutionStage::Haunter {
        return false;
    }
    
    // Check for attack_air_lw animation with frame-specific tongue visibility
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    
    if motion_hash.hash == smash::hash40("attack_air_lw") {
        let motion_frame = MotionModule::frame(boma);
        
        // Show evolving tongue during frames 7-49 (same as normal tongue)
        if motion_frame >= 7.0 && motion_frame <= 49.0 {
            ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING_TONGUE, true);
            println!("[EVOLVING TONGUE] Showing gastly_evolvingtongue during attack_air_lw frames {:.1}", motion_frame);
            return true;
        }
    }
    
    // Check for catch animations (catch_attack, catch_wait) 
    if motion_hash.hash == smash::hash40("catch_attack") || motion_hash.hash == smash::hash40("catch_wait") {
        ModelModule::set_mesh_visibility(boma, *GASTLY_EVOLVING_TONGUE, true);
        println!("[EVOLVING TONGUE] Showing gastly_evolvingtongue during catch animation");
        return true;
    }
    
    false
}

// Check for catch animations that should show long tongue
unsafe fn should_show_tongue_for_catch_animations(boma: *mut BattleObjectModuleAccessor) -> bool {
    let current_motion = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion };
    
    // Show tongue during catch_attack and catch_wait
    motion_hash.hash == smash::hash40("catch_attack") || motion_hash.hash == smash::hash40("catch_wait")
}

// Also update the update_body_and_unique_parts_visibility function to not interfere
pub unsafe fn update_body_and_unique_parts_visibility(boma: *mut BattleObjectModuleAccessor, active_stage: EvolutionStage) {
    let current_status = StatusModule::status_kind(boma);
    
    // CHECK FOR ROLLOUT STATUS FIRST
    if is_in_rollout_status(boma) {
        hide_all_meshes(boma);
        return;
    }
    
    // FIX: Don't interfere with shadowball hold statuses at all
    // Let set_active_eye_mesh handle all shadowball logic
    let is_shadowball_related_status = current_status == 0x1E1 ||    // SPECIAL_N_HOLD (ground/air)
                                       current_status == 0x1E2 ||    // SPECIAL_N_HOLD_MAX (ground/air)
                                       current_status == 0x1E3 ||    // SPECIAL_N_ROLL 
                                       current_status == 0x1E4 ||    // SPECIAL_N_ROLL_AIR
                                       current_status == 0x1E5;      // SPECIAL_N_TURN
    
    if is_shadowball_related_status {
        return; // Don't interfere with shadowball mesh logic
    }
        
    // Always hide all meshes first (both normal and special)
    ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
    ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);

    // Hide all eye expressions
    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    
    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
    
    // Hide all animation-specific meshes
    hide_all_animation_specific_meshes(boma);
    
    // Show appropriate body parts based on stage (only normal body parts, not eyes)
    match active_stage {
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
        }
    }
}

pub unsafe fn update_body_and_unique_parts_visibility_with_enforcement(
    boma: *mut BattleObjectModuleAccessor, 
    active_stage: EvolutionStage, 
    player_state: &PlayerEvolutionState
) {
    // If evolving, let the enhanced eye mesh function handle everything
    if player_state.is_evolving {
        hide_all_evolving_meshes(boma); // Clean slate first
        return; // Let set_active_eye_mesh handle the rest
    }
    
    // Normal non-evolving logic - call your existing function
    update_body_and_unique_parts_visibility(boma, active_stage);
    enforce_gastly_body_hiding(boma, player_state);
}


// Enhanced detect_shadowball_hitbox_state function with official constants
pub unsafe fn detect_shadowball_hitbox_state(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) -> ShadowballState {
    let current_status = StatusModule::status_kind(boma);
    
    let is_shadowball_hold_status = current_status == PURIN_SPECIAL_N_HOLD || 
                                   current_status == PURIN_SPECIAL_N_HOLD_MAX;
    let is_shadowball_roll_air = current_status == PURIN_SPECIAL_N_ROLL_AIR;
    let is_shadowball_roll_ground = current_status == PURIN_SPECIAL_N_ROLL;
    let is_shadowball_roll_turn = current_status == PURIN_SPECIAL_N_TURN;
    let is_shadowball_roll_status = is_shadowball_roll_air || is_shadowball_roll_ground || is_shadowball_roll_turn;
    let is_shadowball_end_status = current_status == PURIN_SPECIAL_N_END;
    
    if is_shadowball_end_status {
        player_state.shadowball_was_sufficiently_charged = false;
        player_state.shadowball_air_charge_count = 0;
        player_state.shadowball_previous_status = -1;
        return ShadowballState::NotActive;
    }
    
    if is_shadowball_hold_status {
        update_shadowball_frame_tracking(player_state, current_status);
        
        let current_motion = MotionModule::motion_kind(boma);
        let is_air_motion = current_motion == SPECIAL_AIR_N_HOLD_MOTION.hash ||
                           current_motion == SPECIAL_AIR_N_HOLD_MAX_MOTION.hash;
        
        if is_air_motion {
            player_state.shadowball_air_charge_count = player_state.shadowball_status_frames;
        }
        
        let official_charge_count = WorkModule::get_float(boma, PURIN_WORK_FLOAT_CHARGE_COUNT);
        let is_max_charge = WorkModule::is_flag(boma, PURIN_FLAG_MAX_FLAG);
        let hold_frames = WorkModule::get_int(boma, PURIN_WORK_INT_HOLD);
        let hold_max_frames = WorkModule::get_int(boma, PURIN_WORK_INT_HOLD_MAX);
        
        let frame_threshold = get_shadowball_frame_threshold(boma);
        let our_frame_threshold_met = player_state.shadowball_status_frames > frame_threshold;
        let official_charge_threshold_met = official_charge_count > frame_threshold as f32;
        let official_hold_threshold_met = hold_frames > frame_threshold || hold_max_frames > 0;
        
        let charge_threshold_met = our_frame_threshold_met || 
                                  official_charge_threshold_met || 
                                  official_hold_threshold_met || 
                                  is_max_charge;
        
        if charge_threshold_met {
            player_state.shadowball_was_sufficiently_charged = true;
        }
        
        if is_air_motion && player_state.shadowball_air_charge_count < 28 && official_charge_count < 28.0 && !is_max_charge {
            return ShadowballState::ChargingBelowThreshold;
        }
        
        if charge_threshold_met {
            return ShadowballState::ActiveFrameBased;
        }
        
        return ShadowballState::ChargingBelowThreshold;
    }
    
    // NEW: Enhanced rollout detection with hitbox-based invisibility
    if is_shadowball_roll_status {
    let has_active_hitbox = is_rollout_hitbox_active(boma);
    
        if is_shadowball_roll_air {
            // For air roll, ONLY use hitbox detection - ignore frame thresholds
            if has_active_hitbox {
                return ShadowballState::RegularRolloutWithHitbox; // Always invisible when hitbox active
            } else {
                return ShadowballState::RegularRollout; // Always visible when no hitbox
            }
        }
        
        // Ground roll logic - same hitbox-only approach
        if is_shadowball_roll_ground {
            if has_active_hitbox {
                return ShadowballState::RegularRolloutWithHitbox;
            } else {
                return ShadowballState::RegularRollout;
            }
        }
        
        // Roll turn - always keep visible (no hitboxes in this status)
        if is_shadowball_roll_turn {
            return ShadowballState::RegularRollout; // Always visible during turn
        }
    }

    if !is_shadowball_hold_status && !is_shadowball_roll_status && !is_shadowball_end_status {
        player_state.shadowball_was_sufficiently_charged = false;
        player_state.shadowball_air_charge_count = 0;
        player_state.shadowball_previous_status = -1;
    }

    ShadowballState::NotActive
}

unsafe fn is_rollout_hitbox_active(boma: *mut BattleObjectModuleAccessor) -> bool {
    // Primary detection methods for rollout hitboxes
    let attack_active = AttackModule::is_attack(boma, 0, false) ||
                       AttackModule::is_attack(boma, 1, false) ||
                       AttackModule::is_attack(boma, 2, false);
    
    let infliction_active = AttackModule::is_infliction_status(boma, 0);
    let occur_active = AttackModule::is_attack_occur(boma);
    
    let has_hitbox = attack_active || infliction_active || occur_active;
    
    // Optional debug to verify detection
    if has_hitbox {
    //    println!("[HITBOX DEBUG] Active rollout hitbox detected - Attack:{}, Infliction:{}, Occur:{}", 
    //            attack_active, infliction_active, occur_active);
    }
    
    has_hitbox
}


// NEW: Helper function to update shadowball frame tracking with status history
unsafe fn update_shadowball_frame_tracking(player_state: &mut PlayerEvolutionState, current_status: i32) {
    // Store previous status before updating
    if player_state.is_in_shadowball_status && player_state.last_shadowball_status != current_status {
        player_state.shadowball_previous_status = player_state.last_shadowball_status;
    }
    
    if !player_state.is_in_shadowball_status {
        player_state.is_in_shadowball_status = true;
        player_state.shadowball_status_frames = 1;
        player_state.last_shadowball_status = current_status;
        player_state.shadowball_previous_status = -1; // No previous status for new sequence
    } else {
        let is_valid_transition = (player_state.last_shadowball_status == 0x1E1 && current_status == 0x1E2) ||
                                 (player_state.last_shadowball_status == current_status);
        
        if is_valid_transition {
            player_state.shadowball_status_frames += 1;
            player_state.last_shadowball_status = current_status;
        } else {
            // Status changed - store previous and reset counter
            player_state.shadowball_previous_status = player_state.last_shadowball_status;
            player_state.shadowball_status_frames = 1;
            player_state.last_shadowball_status = current_status;
        }
    }
}

// NEW: Helper function to get frame threshold based on motion
pub unsafe fn get_shadowball_frame_threshold(boma: *mut BattleObjectModuleAccessor) -> i32 {
    let current_motion = MotionModule::motion_kind(boma);
    let is_air_motion = current_motion == SPECIAL_AIR_N_HOLD_MOTION.hash ||
                       current_motion == SPECIAL_AIR_N_HOLD_MAX_MOTION.hash;
    
    if is_air_motion {
        SHADOWBALL_MESH_DELAY_FRAMES_AIR
    } else {
        SHADOWBALL_MESH_DELAY_FRAMES_GROUND
    }
}

// NEW: Handle shadowball roll transition logic
unsafe fn handle_shadowball_roll_transition(player_state: &mut PlayerEvolutionState, current_status: i32) -> ShadowballState {
    // Update status tracking for consistency
    let _previous_status = player_state.last_shadowball_status;
    player_state.last_shadowball_status = current_status;
    
    // IMPORTANT: During ANY roll status, we should NEVER show the shadowball mesh
    // The character should be completely invisible during rolls
    // We preserve the frame count for tracking purposes but don't use it for mesh visibility
    
    ShadowballState::RegularRollout // Always return regular rollout - no shadowball mesh during rolls
}

// In visuals.rs - Updated set_active_eye_mesh function
pub unsafe fn set_active_eye_mesh(
    boma: *mut BattleObjectModuleAccessor, 
    player_state: &mut PlayerEvolutionState, 
    game_state_expression_override: Option<Hash40>
) {
    let current_status = StatusModule::status_kind(boma);
    let current_motion = MotionModule::motion_kind(boma);
    
    // First, always ensure all animation-specific meshes are hidden
    hide_all_animation_specific_meshes(boma);
    hide_all_evolving_meshes(boma); // Also hide evolving meshes initially
    
    // PRIORITY 1: Enhanced shadowball detection (highest priority - can override evolution)
    let shadowball_state = detect_shadowball_hitbox_state(boma, player_state);
    
    // Handle shadowball states FIRST (even during evolution)
    match shadowball_state {
        ShadowballState::ActiveWithHitbox | ShadowballState::ActiveFrameBased => {
            let is_hold_status = current_status == 0x1E1 || current_status == 0x1E2;
            
            if is_hold_status {
                // Show shadowball mesh (evolving version if evolving, normal if not)
                let shadow_ball_mesh = if player_state.is_evolving {
                    match (player_state.stage, player_state.evolution_target_stage) {
                        (EvolutionStage::Gastly, EvolutionStage::Haunter) => *GASTLY_EVOLVING_SHADOWBALL,
                        (EvolutionStage::Haunter, EvolutionStage::Gengar) => *HAUNTER_EVOLVING_SHADOWBALL,
                        _ => match player_state.stage {
                            EvolutionStage::Gastly => *GASTLY_SHADOWBALL,
                            EvolutionStage::Haunter => *HAUNTER_SHADOWBALL,
                            EvolutionStage::Gengar => *GENGAR_SHADOWBALL,
                        }
                    }
                } else {
                    match player_state.stage {
                        EvolutionStage::Gastly => *GASTLY_SHADOWBALL,
                        EvolutionStage::Haunter => *HAUNTER_SHADOWBALL,
                        EvolutionStage::Gengar => *GENGAR_SHADOWBALL,
                    }
                };
                
                ModelModule::set_mesh_visibility(boma, shadow_ball_mesh, true);
                hide_all_normal_meshes_and_eyes(boma);
                return;
            }
        },
        
        // ANY rollout with active hitbox = completely invisible (no evolving mesh)
        ShadowballState::ChargedRolloutWithHitbox | 
        ShadowballState::RegularRolloutWithHitbox | 
        ShadowballState::AirToGroundRolloutWithHitbox => {
            hide_all_meshes(boma);
            return;
        },
        
        // Rollout without hitbox OR turn status = visible (evolving if evolving, normal if not)
        ShadowballState::ChargedRollout |
        ShadowballState::RegularRollout |
        ShadowballState::AirToGroundRollout => {
            // SPECIAL CASE: TURN status should show model (evolving if evolving)
            if current_status == PURIN_SPECIAL_N_TURN {
                if player_state.is_evolving {
                    // Show evolving mesh during turn status
                    let evolving_main = match (player_state.stage, player_state.evolution_target_stage) {
                        (EvolutionStage::Gastly, EvolutionStage::Haunter) => *GASTLY_EVOLVING,
                        (EvolutionStage::Haunter, EvolutionStage::Gengar) => *HAUNTER_EVOLVING,
                        _ => {
                            restore_normal_body_parts(boma, player_state);
                            enforce_gastly_body_hiding(boma, player_state);
                            handle_animation_specific_tongue_visibility(boma, player_state);
                            show_appropriate_eye_expression(boma, player_state, game_state_expression_override);
                            return;
                        }
                    };
                    ModelModule::set_mesh_visibility(boma, evolving_main, true);
                    
                    // Hide all eyes during evolution turn
                    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { 
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
                    }
                    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { 
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
                    }
                    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { 
                        ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
                    }
                    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
                    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
                    return;
                } else {
                    // Normal turn status - show normal model
                    restore_normal_body_parts(boma, player_state);
                    enforce_gastly_body_hiding(boma, player_state);
                    handle_animation_specific_tongue_visibility(boma, player_state);
                    show_appropriate_eye_expression(boma, player_state, game_state_expression_override);
                    return;
                }
            } else {
                // Regular rollout without hitbox - normal visibility logic
                restore_normal_body_parts(boma, player_state);
                enforce_gastly_body_hiding(boma, player_state);
                handle_animation_specific_tongue_visibility(boma, player_state);
                show_appropriate_eye_expression(boma, player_state, game_state_expression_override);
                return;
            }
        },
        
        ShadowballState::ChargingBelowThreshold |
        ShadowballState::TransitionKeepModel => {
            // Continue to normal processing
        },
        
        ShadowballState::NotActive => {
            // Reset shadowball tracking if completely out of shadowball statuses
            if player_state.is_in_shadowball_status || player_state.shadowball_status_frames > 0 {
                player_state.is_in_shadowball_status = false;
                player_state.shadowball_status_frames = 0;
                player_state.last_shadowball_status = -1;
                player_state.shadowball_previous_status = -1;
                player_state.shadowball_air_charge_count = 0;
            }
        }
    }
    
    // PRIORITY 2: Check if we should show evolving meshes (only if not handled by shadowball logic above)
    if player_state.is_evolving {
        if show_evolving_meshes_for_animation(boma, player_state, current_motion) {
            // Hide all normal meshes when showing evolving meshes
            hide_all_normal_meshes_and_eyes(boma);
            
            // CRITICAL FIX: Hide ALL eye expressions during evolution - no blinking!
            for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { 
                ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
            }
            for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { 
                ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
            }
            for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { 
                ModelModule::set_mesh_visibility(boma, *eye_hash, false); 
            }
            for eye_hash in PURIN_VANILLA_EYES_TO_HIDE.iter() {
                ModelModule::set_mesh_visibility(boma, *eye_hash, false);
            }
            
            // Hide iris as well during evolution
            ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
            ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
            
            // Hide all body parts (they'll be replaced by evolving mesh)
            ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
            ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, false);
            ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, false);
            ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
            ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
            ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
            
            return; // Exit early - ONLY evolving mesh should be visible, nothing else
        }
    }
    
    // PRIORITY 3: Check for other special animation meshes (non-shadowball)
    let using_special_mesh = check_and_show_other_animation_meshes(boma, player_state);
    
    if using_special_mesh {
        hide_all_normal_meshes_and_eyes(boma);
        return;
    }
    
    // PRIORITY 4: Default - Restore normal body parts and show eyes
    restore_normal_body_parts(boma, player_state);
    enforce_gastly_body_hiding(boma, player_state);
    handle_animation_specific_tongue_visibility(boma, player_state);
    show_appropriate_eye_expression(boma, player_state, game_state_expression_override);
}

// NEW: Handle animation-specific tongue visibility
unsafe fn handle_animation_specific_tongue_visibility(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) {
    // Check for gengar_tongue_normal animations (Gengar only, non-FS)
    if should_show_gengar_tongue_normal(boma, player_state) {
        ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, true);
        return;
    }
    
    // Check for attack_air_lw frame-specific tongue (frames 7-49)
    if should_show_tongue_for_attack_air_lw(boma, player_state) {
        match player_state.stage {
            EvolutionStage::Gastly => {
                ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, true);
            }
            EvolutionStage::Gengar => {
                ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, true);
            }
            _ => {} // Haunter doesn't have tongue for this animation
        }
        return;
    }
    
    // Check for catch animations (catch_attack, catch_wait)
    if should_show_tongue_for_catch_animations(boma) {
        match player_state.stage {
            EvolutionStage::Gastly => {
                ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, true);
            }
            EvolutionStage::Gengar => {
                ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, true);
            }
            _ => {} // Haunter doesn't have tongue for catch animations
        }
    }
}

// NEW: Separate function to hide all normal meshes and eyes
unsafe fn hide_all_normal_meshes_and_eyes(boma: *mut BattleObjectModuleAccessor) {
    // Hide all normal body meshes
    ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
    
    // Hide all eyes
    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
}

// NEW: Restore normal body parts based on current stage
unsafe fn restore_normal_body_parts(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) {
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
        }
    }
}

// NEW: Show appropriate eye expression (extracted from original logic)
unsafe fn show_appropriate_eye_expression(
    boma: *mut BattleObjectModuleAccessor, 
    player_state: &PlayerEvolutionState, 
    game_state_expression_override: Option<Hash40>
) {
    let (current_stage_expressions, default_mesh_to_use, half_mesh_to_use, full_mesh_to_use) = match player_state.stage {
        EvolutionStage::Gastly => {
            for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            (GASTLY_EYE_EXPRESSIONS.as_slice(), *GASTLY_EYE_N, *GASTLY_EYE_HALFBLINK1, *GASTLY_EYE_BLINK)
        }
        EvolutionStage::Haunter => {
            for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            (HAUNTER_EYELID_EXPRESSIONS.as_slice(), *HAUNTER_EYE_N, *HAUNTER_EYE_HALFBLINK1, *HAUNTER_EYE_BLINK)
        }
        EvolutionStage::Gengar => {
            for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
            (GENGAR_EYELID_EXPRESSIONS.as_slice(), *GENGAR_EYE_N, *GENGAR_EYE_HALFBLINK1, *GENGAR_EYE_BLINK)
        }
    };

    for eye_hash in current_stage_expressions.iter() {
        ModelModule::set_mesh_visibility(boma, *eye_hash, false);
    }
    
    let eye_to_show = if let Some(override_eye_hash) = game_state_expression_override {
        override_eye_hash
    } else {
        match player_state.blink_phase {
            BlinkPhase::Open => default_mesh_to_use,
            BlinkPhase::HalfClose | BlinkPhase::HalfOpen => half_mesh_to_use,
            BlinkPhase::FullClose => full_mesh_to_use,
        }
    };

    // Apply weakened state system here
    let final_eye_to_show = handle_weakened_state_isolated(boma, player_state, eye_to_show, player_state.current_frame);

    ModelModule::set_mesh_visibility(boma, final_eye_to_show, true);
}


// NEW: Check for non-shadowball animation meshes (refactored from original)
unsafe fn check_and_show_other_animation_meshes(boma: *mut BattleObjectModuleAccessor, player_state: &PlayerEvolutionState) -> bool {
    let current_motion = MotionModule::motion_kind(boma);
    
    // Check for squat_wait animation (floor shadow)
    if current_motion == SQUAT_WAIT_MOTION.hash {
        let floor_shadow_mesh = match player_state.stage {
            EvolutionStage::Gastly => *GASTLY_FLOORSHADOW,
            EvolutionStage::Haunter => *HAUNTER_FLOORSHADOW,
            EvolutionStage::Gengar => *GENGAR_FLOORSHADOW,
        };
        ModelModule::set_mesh_visibility(boma, floor_shadow_mesh, true);
        return true;
    }
    
    // Check for appeal_lw animations (ghost)
    if current_motion == APPEAL_LW_L_MOTION.hash ||
       current_motion == APPEAL_LW_R_MOTION.hash {
        ModelModule::set_mesh_visibility(boma, *GHOST, true);
        return true;
    }
    
    // Check for run animation (gengar only)
    if current_motion == RUN_MOTION.hash && player_state.stage == EvolutionStage::Gengar {
        ModelModule::set_mesh_visibility(boma, *GENGAR_RUN, true);
        return true;
    }
    
    false
}

pub unsafe fn handle_final_smash_model_swap(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) {
    let is_final_smash_active_flag = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
    let current_motion = MotionModule::motion_kind(boma);
    let current_motion_hash = Hash40 { hash: current_motion };
    
    let is_in_fs_attack_animation = current_motion == FINAL_SMASH_MOTION_HASH.hash ||
                                    current_motion == FINAL_SMASH_START_R_MOTION_HASH.hash;
    
    // DEBUG: Log final smash state
    static mut LAST_FS_STATE: [bool; 8] = [false; 8];
    static mut LAST_MOTION: [u64; 8] = [0; 8];
    static mut LAST_FS_FORM: [bool; 8] = [false; 8];
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    
    if entry_id < 8 {
        if LAST_FS_STATE[entry_id] != is_final_smash_active_flag || 
           LAST_MOTION[entry_id] != current_motion || 
           LAST_FS_FORM[entry_id] != player_state.is_in_final_smash_form {
            println!("[FS DEBUG] Entry {} - FS Flag: {}, Motion: {:#x}, In Attack Anim: {}, Player FS Form: {}", 
                    entry_id, is_final_smash_active_flag, current_motion, is_in_fs_attack_animation, player_state.is_in_final_smash_form);
            
            // AGGRESSIVE: Force cleanup on motion change if in final smash form and not in attack animation
            if player_state.is_in_final_smash_form && !is_in_fs_attack_animation && LAST_MOTION[entry_id] != current_motion {
                println!("[FS DEBUG] Motion changed while in FS form and not in attack - forcing cleanup!");
                
                // Force cleanup immediately
                EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
                ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
                ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
                ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
                
                player_state.is_in_final_smash_form = false;
                
                // Force normal visibility for current stage
                match player_state.stage {
                    EvolutionStage::Gastly => {
                        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, true);
                        ModelModule::set_mesh_visibility(boma, *GASTLY_EYE_N, true);
                    },
                    EvolutionStage::Haunter => {
                        ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, true);
                        ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, true);
                        ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, true);
                        ModelModule::set_mesh_visibility(boma, *HAUNTER_EYE_N, true);
                    },
                    EvolutionStage::Gengar => {
                        ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, true);
                        ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, true);
                        ModelModule::set_mesh_visibility(boma, *GENGAR_EYE_N, true);
                    }
                }
                
                println!("[FS DEBUG] Forced cleanup complete!");
            }
            
            LAST_FS_STATE[entry_id] = is_final_smash_active_flag;
            LAST_MOTION[entry_id] = current_motion;
            LAST_FS_FORM[entry_id] = player_state.is_in_final_smash_form;
        }
    }
    
    // NEW: Kill sys_final_aura2 effects during ALL final smash motions
    let fs_motions = [
        smash::hash40("final_start_r"),
        smash::hash40("final"),
        smash::hash40("final_end_r"),
        smash::hash40("final_air_start_r"),
        smash::hash40("final_air"),
        smash::hash40("final_air_end_r"),
    ];
    
    if fs_motions.contains(&current_motion_hash.hash) {
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
    }

    if is_final_smash_active_flag && is_in_fs_attack_animation && !player_state.is_in_final_smash_form {
        // Entering Final Smash form
        println!("[FS DEBUG] Entering final smash form");
        if player_state.stage == EvolutionStage::Gengar {
            // Hide all animation-specific meshes during FS
            hide_all_animation_specific_meshes(boma);
            
            if player_state.mega_gengar_form_active {
                // Hide normal Gengar parts
                ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false); 
                for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
                ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
                // Show Mega Gengar
                ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, true);
                player_state.is_in_final_smash_form = true;
                println!("[FS DEBUG] Activated Mega Gengar form");
            } else if player_state.giga_gengar_form_active {
                // Hide normal Gengar parts
                ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false); 
                for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
                ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
                // Show Giga Gengar
                ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, true);
                player_state.is_in_final_smash_form = true;
                println!("[FS DEBUG] Activated Giga Gengar form");
            }
        }
    } else if (!is_in_fs_attack_animation && player_state.is_in_final_smash_form) || 
              (!is_final_smash_active_flag && player_state.is_in_final_smash_form) { 
        // Exiting Final Smash form - AGGRESSIVE CLEANUP
        
        println!("[FINAL SMASH] Starting aggressive cleanup - Current stage: {:?}", player_state.stage);
        
        // STEP 1: Kill any remaining aura effects
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
        
        // STEP 2: AGGRESSIVELY hide ALL possible meshes that could be visible
        ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, false);
        ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, false);
        ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, false);
        ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
        ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
        ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
        ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
        ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
        
        // STEP 3: Hide ALL eye expressions from ALL stages
        for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() {
            ModelModule::set_mesh_visibility(boma, *eye_hash, false);
        }
        for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() {
            ModelModule::set_mesh_visibility(boma, *eye_hash, false);
        }
        for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() {
            ModelModule::set_mesh_visibility(boma, *eye_hash, false);
        }
        
        // STEP 4: Hide all animation-specific meshes
        hide_all_animation_specific_meshes(boma);
        
        // STEP 5: Reset the final smash flag
        player_state.is_in_final_smash_form = false;
        
        // STEP 6: Force a complete visibility reset using existing function
        update_body_and_unique_parts_visibility(boma, player_state.stage);
        
        // STEP 7: Set the correct eye expression manually as backup
        let eye_to_show = match player_state.stage {
            EvolutionStage::Gastly => {
                match player_state.blink_phase {
                    crate::gastly::player_state::BlinkPhase::Open => *GASTLY_EYE_N,
                    crate::gastly::player_state::BlinkPhase::HalfClose | 
                    crate::gastly::player_state::BlinkPhase::HalfOpen => *GASTLY_EYE_HALFBLINK1,
                    crate::gastly::player_state::BlinkPhase::FullClose => *GASTLY_EYE_BLINK,
                }
            },
            EvolutionStage::Haunter => {
                match player_state.blink_phase {
                    crate::gastly::player_state::BlinkPhase::Open => *HAUNTER_EYE_N,
                    crate::gastly::player_state::BlinkPhase::HalfClose | 
                    crate::gastly::player_state::BlinkPhase::HalfOpen => *HAUNTER_EYE_HALFBLINK1,
                    crate::gastly::player_state::BlinkPhase::FullClose => *HAUNTER_EYE_BLINK,
                }
            },
            EvolutionStage::Gengar => {
                match player_state.blink_phase {
                    crate::gastly::player_state::BlinkPhase::Open => *GENGAR_EYE_N,
                    crate::gastly::player_state::BlinkPhase::HalfClose | 
                    crate::gastly::player_state::BlinkPhase::HalfOpen => *GENGAR_EYE_HALFBLINK1,
                    crate::gastly::player_state::BlinkPhase::FullClose => *GENGAR_EYE_BLINK,
                }
            }
        };
        
        ModelModule::set_mesh_visibility(boma, eye_to_show, true);
        
        println!("[FINAL SMASH] Aggressive cleanup complete - Restored {:?} with eye: {:#x}", 
                player_state.stage, eye_to_show.hash);
    }
}

pub unsafe fn hide_all_animation_specific_meshes(boma: *mut BattleObjectModuleAccessor) {
    // Hide shadow floor meshes
    ModelModule::set_mesh_visibility(boma, *GASTLY_FLOORSHADOW, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_FLOORSHADOW, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_FLOORSHADOW, false);
    
    // Hide shadow ball meshes
    ModelModule::set_mesh_visibility(boma, *GASTLY_SHADOWBALL, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_SHADOWBALL, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_SHADOWBALL, false);
    
    // Hide ghost mesh
    ModelModule::set_mesh_visibility(boma, *GHOST, false);
    
    // Hide gengar run mesh
    ModelModule::set_mesh_visibility(boma, *GENGAR_RUN, false);
    
    // Hide all tongue meshes (they'll be shown by animation-specific logic if needed)
    ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
    
    // NEW: Hide all evolving meshes
    hide_all_evolving_meshes(boma);
}

// Enhanced mesh management for evolving state
unsafe fn show_evolving_meshes_for_animation(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    current_motion: u64
) -> bool {
    if !player_state.is_evolving {
        return false;
    }

    let evolution_stage = player_state.stage;
    let target_stage = player_state.evolution_target_stage;
    
    // Determine which evolving meshes to use based on current evolution
    let (evolving_main, evolving_floorshadow, evolving_tongue) = match (evolution_stage, target_stage) {
        (EvolutionStage::Gastly, EvolutionStage::Haunter) => {
            (*GASTLY_EVOLVING, *GASTLY_EVOLVING_FLOORSHADOW, *GASTLY_EVOLVING_TONGUE)
        },
        (EvolutionStage::Haunter, EvolutionStage::Gengar) => {
            // Haunter doesn't have tongue, so use a dummy value that won't be shown
            (*HAUNTER_EVOLVING, *HAUNTER_EVOLVING_FLOORSHADOW, *GASTLY_TONGUE) // Dummy tongue
        },
        _ => return false, // Invalid evolution combination
    };

    // Check for special animation cases
    let motion_hash = Hash40 { hash: current_motion };
    let current_status = StatusModule::status_kind(boma);
    
    // IMPORTANT: Don't show evolving meshes during rollout statuses 
    // (shadowball logic in set_active_eye_mesh handles those cases)
    if current_status == PURIN_SPECIAL_N_ROLL || current_status == PURIN_SPECIAL_N_ROLL_AIR {
        return false; // Let shadowball logic handle rollout visibility
    }
    
    // Don't show during shadowball hold statuses either
    // (shadowball logic handles evolving shadowball mesh)
    if current_status == PURIN_SPECIAL_N_HOLD || current_status == PURIN_SPECIAL_N_HOLD_MAX {
        return false; // Let shadowball logic handle this
    }
    
    // Check for squat_wait animation (floor shadow)
    if motion_hash.hash == SQUAT_WAIT_MOTION.hash {
        ModelModule::set_mesh_visibility(boma, evolving_floorshadow, true);
        println!("[EVOLVING MESH] Showing evolving floor shadow during squat_wait");
        return true;
    }
    
    // Check for tongue animations (only for Gastly → Haunter evolution)
    if evolution_stage == EvolutionStage::Gastly && target_stage == EvolutionStage::Haunter {
        // Check for attack_air_lw animation with frame-specific tongue visibility
        if motion_hash.hash == smash::hash40("attack_air_lw") {
            let motion_frame = MotionModule::frame(boma);
            
            // Show evolving tongue + main mesh during frames 7-49
            if motion_frame >= 7.0 && motion_frame <= 49.0 {
                ModelModule::set_mesh_visibility(boma, evolving_main, true);
                ModelModule::set_mesh_visibility(boma, evolving_tongue, true);
                println!("[EVOLVING MESH] Showing main + tongue during attack_air_lw frame {:.1}", motion_frame);
                return true;
            }
        }
        
        // Check for catch animations (catch_attack, catch_wait) 
        if motion_hash.hash == smash::hash40("catch_attack") || motion_hash.hash == smash::hash40("catch_wait") {
            ModelModule::set_mesh_visibility(boma, evolving_main, true);
            ModelModule::set_mesh_visibility(boma, evolving_tongue, true);
            println!("[EVOLVING MESH] Showing main + tongue during catch animation");
            return true;
        }
    }
    
    // Default case - show main evolving mesh for all other animations
    ModelModule::set_mesh_visibility(boma, evolving_main, true);
    println!("[EVOLVING MESH] Showing main evolving mesh - default case");
    true
}

// DEPRECATED: Old function kept for compatibility, but now uses new detection
pub unsafe fn check_and_show_animation_specific_meshes(boma: *mut BattleObjectModuleAccessor, player_state: &mut PlayerEvolutionState) -> bool {
    // This function is now handled by the new shadowball detection system
    // Keep for any external calls, but redirect to new logic
    let shadowball_state = detect_shadowball_hitbox_state(boma, player_state);
    
    match shadowball_state {
        ShadowballState::ActiveWithHitbox | ShadowballState::ActiveFrameBased => true,
        ShadowballState::RegularRollout => true,
        _ => check_and_show_other_animation_meshes(boma, player_state)
    }
}

// Check if player is in rollout status
pub unsafe fn is_in_rollout_status(boma: *mut BattleObjectModuleAccessor) -> bool {
    let current_status = StatusModule::status_kind(boma);
    current_status == 0x1E3 || current_status == 0x1E4 || current_status == 0x1E5   // SPECIAL_N_ROLL | SPECIAL_N_ROLL_AIR | SPECIAL_N_TURN
}

// ADD this bomb detection function to visuals.rs:
unsafe fn handle_bomb_detection_in_visuals(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState
) {
    static mut BOMB_STATE: [SimpleHitState; 8] = [SimpleHitState::new(); 8];
    
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if entry_id >= 8 { return; }
    
    let hit_state = &mut BOMB_STATE[entry_id];
    
    // Simple hit detection
    let currently_hitting = AttackModule::is_infliction_status(boma, 0) ||
                           AttackModule::is_attack_occur(boma) ||
                           AttackModule::is_attack(boma, 0, false);
    
    // Fresh hit with cooldown
    let fresh_hit = currently_hitting && 
                   !hit_state.was_hitting && 
                   (player_state.current_frame - hit_state.last_bomb_frame >= 15);
    
    if fresh_hit {
        println!("[VISUALS BOMB] Hit detected at frame {}", player_state.current_frame);
        
        // Get enemy position (simplified)
        let mut enemy_pos = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        ModelModule::joint_global_position(boma, Hash40::new("top"), &mut enemy_pos, true);
        
        // Offset forward for enemy location
        let facing = PostureModule::lr(boma);
        enemy_pos.x += facing * 3.0;
        enemy_pos.y += 1.5;
        
        // Clean up old bombs
        EffectModule::kill_kind(boma, Hash40::new("mewtwo_shadowball_bomb"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_bomb_a"), false, true);
        
        // Spawn bomb
        let bomb_rot = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let bomb_handle = EffectModule::req(
            boma,
            Hash40::new("mewtwo_shadowball_bomb").into(),
            &enemy_pos,
            &bomb_rot,
            2.5,
            60, // Lifetime frames
            -1,
            false,
            0
        );
        
        // Backup effect
        let backup_handle = EffectModule::req(
            boma,
            Hash40::new("sys_bomb_a").into(),
            &enemy_pos,
            &bomb_rot,
            1.8,
            60,
            -1,
            false,
            0
        );
        
        if bomb_handle != u64::MAX || backup_handle != u64::MAX {
            hit_state.last_bomb_frame = player_state.current_frame;
            println!("[VISUALS BOMB] Spawned bomb effect");
        }
    }
    
    hit_state.was_hitting = currently_hitting;
}

// SINGLE function that handles everything - no external calls can interfere
pub unsafe fn handle_weakened_state_isolated(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    eye_expression: Hash40,
    current_frame: i32
) -> Hash40 {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
    if entry_id >= 8 { return eye_expression; }
    
    // DEBUG: Check if flag was mysteriously reset
    let was_spawned_last_frame = WEAKENED_SPAWNED[entry_id];
    let last_frame_checked = DEBUG_LAST_FRAME_CHECKED[entry_id];
    
    if was_spawned_last_frame && last_frame_checked == (current_frame - 1) {
        // Flag should still be true from last frame
        if !WEAKENED_SPAWNED[entry_id] {
            println!("[WEAKENED BUG] Flag was reset between frames {} and {}!", last_frame_checked, current_frame);
        }
    }
    DEBUG_LAST_FRAME_CHECKED[entry_id] = current_frame;
    
    let current_damage = DamageModule::damage(boma, 0);
    let last_damage = WEAKENED_LAST_DAMAGE[entry_id];
    let already_spawned = WEAKENED_SPAWNED[entry_id];
    
    // Check if should be weakened (copy your exact logic)
    let should_be_weakened = {
        if current_damage < 150.0 { false }
        else {
            let current_motion = MotionModule::motion_kind(boma);
            let current_status = StatusModule::status_kind(boma);
            
            // Exclude squat_wait animation
            if current_motion == smash::hash40("squat_wait") { false }
            // Exclude neutral special statuses
            else if current_status == 0x1E1 || current_status == 0x1E2 || 
                    current_status == 0x1E3 || current_status == 0x1E4 || 
                    current_status == 0x1E5 || current_status == 0x1E6 { false }
            // Exclude final smash
            else if WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL) || 
                    player_state.is_in_final_smash_form { false }
            else { true }
        }
    };
    
    // Detect significant damage drop (training mode reset)
    let damage_dropped = last_damage >= 150.0 && current_damage < 150.0;
    
    // Handle effect spawning/cleanup
    if damage_dropped && already_spawned {
        // Training mode damage drop - kill effect
        EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_weak"), false, true);
        WEAKENED_SPAWNED[entry_id] = false;
        println!("[WEAKENED ISOLATED] Killed due to damage drop: {:.1} -> {:.1}", last_damage, current_damage);
    }
    else if should_be_weakened && !already_spawned {
        // Need to spawn effect
        let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
        
        let handle = EffectModule::req_follow(
            boma,
            Hash40::new("rosetta_tico_weak"),
            Hash40::new("body"),
            &position_offset,
            &rotation_vector,
            1.0,
            true, 0x40000, 0, -1, 0, 0, false, false
        ) as u32;
        
        if handle != u64::MAX as u32 && handle != 0 {
            WEAKENED_SPAWNED[entry_id] = true;
            println!("[WEAKENED ISOLATED] Spawned effect - Handle: {} - Frame: {} - Flag set to TRUE", handle, current_frame);
        }
    }
    else if !should_be_weakened && already_spawned {
        // No longer weakened - kill effect
        EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_weak"), false, true);
        WEAKENED_SPAWNED[entry_id] = false;
        println!("[WEAKENED ISOLATED] Killed - no longer weakened");
    }
    
    // Handle death cleanup (low damage after having high damage)
    if current_damage <= 5.0 && last_damage >= 50.0 && already_spawned {
        EffectModule::kill_kind(boma, Hash40::new("rosetta_tico_weak"), false, true);
        WEAKENED_SPAWNED[entry_id] = false;
        println!("[WEAKENED ISOLATED] Killed on death/respawn");
    }
    
    // Update damage tracking
    WEAKENED_LAST_DAMAGE[entry_id] = current_damage;
    
    // Return appropriate eye expression
    if should_be_weakened {
        // Only replace normal expressions with halfblink
        let should_replace = match player_state.stage {
            EvolutionStage::Gastly => eye_expression.hash == smash::hash40("gastly_eyen"),
            EvolutionStage::Haunter => eye_expression.hash == smash::hash40("haunter_eyen"),
            EvolutionStage::Gengar => eye_expression.hash == smash::hash40("gengar_eyen"),
        };
        
        if should_replace {
            match player_state.stage {
                EvolutionStage::Gastly => Hash40::new("gastly_eye_halfblink1"),
                EvolutionStage::Haunter => Hash40::new("haunter_eye_halfblink1"),
                EvolutionStage::Gengar => Hash40::new("gengar_eye_halfblink1"),
            }
        } else {
            eye_expression
        }
    } else {
        eye_expression
    }
}

// Simple cleanup function for emergencies only
pub unsafe fn emergency_cleanup_weakened(entry_id: u32) {
    let entry_idx = entry_id as usize;
    if entry_idx >= 8 { return; }
    
    if WEAKENED_SPAWNED[entry_idx] {
        let fighter_boma = smash::app::sv_battle_object::module_accessor(entry_id);
        if !fighter_boma.is_null() {
            EffectModule::kill_kind(fighter_boma, Hash40::new("rosetta_tico_weak"), false, true);
        }
        WEAKENED_SPAWNED[entry_idx] = false;
        WEAKENED_LAST_DAMAGE[entry_idx] = 0.0;
        println!("[WEAKENED ISOLATED] Emergency cleanup for entry {}", entry_id);
    }
}

// Hide all meshes (for rollout)
pub unsafe fn hide_all_meshes(boma: *mut BattleObjectModuleAccessor) {
    // Hide all normal body meshes
    ModelModule::set_mesh_visibility(boma, *GASTLY_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GASTLY_TONGUE, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_BODY, false);
    ModelModule::set_mesh_visibility(boma, *HAUNTER_HANDS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_LONG, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_TONGUE_NORMAL, false);
    ModelModule::set_mesh_visibility(boma, *MEGA_GENGAR_BODY, false);
    ModelModule::set_mesh_visibility(boma, *GIGA_GENGAR_BODY, false);

    // Hide all eye expressions
    for eye_hash in GASTLY_EYE_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in PURIN_VANILLA_EYES_TO_HIDE.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in HAUNTER_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    for eye_hash in GENGAR_EYELID_EXPRESSIONS.iter() { ModelModule::set_mesh_visibility(boma, *eye_hash, false); }
    
    ModelModule::set_mesh_visibility(boma, *HAUNTER_IRIS, false);
    ModelModule::set_mesh_visibility(boma, *GENGAR_IRIS, false);
    
    // Hide all animation-specific meshes
    hide_all_animation_specific_meshes(boma);
} 