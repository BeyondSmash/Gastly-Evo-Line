// src/gastly/effects.rs

use smash::app::lua_bind::{StatusModule, MotionModule, EffectModule, 
    WorkModule, AttackModule, SoundModule};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::{Hash40, Vector3f};
use smash::lib::lua_const::*;
use smash_script::macros;
use smash::lua2cpp::L2CFighterCommon;

// Import from our modules
use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};
use crate::gastly::constants::*;
use crate::gastly::visuals::{detect_shadowball_hitbox_state, ShadowballState};

use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// GAS AURA PERSISTENT EFFECT SETTINGS
pub const GASTLY_AURA_HANDLE_WORK_ID: i32 = 0x50000020; // Custom work ID for storing handle

// NEW: Easy-to-edit Gastly Aura Settings
#[derive(Clone, Copy)]
pub struct GastlyAuraSettings {
    pub effect_name: &'static str,
    pub bone: &'static str,
    pub position_x: f32,    // NEW: X position offset
    pub position_y: f32,    // NEW: Y position offset  
    pub position_z: f32,    // NEW: Z position offset
    pub scale: f32,
    pub rotation_x: f32,
    pub rotation_y: f32, 
    pub rotation_z: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
    pub alpha: f32,
    pub rate: f32,
}

// EASY EDIT SECTION - Change any parameter here
pub const GASTLY_AURA_SETTINGS: GastlyAuraSettings = GastlyAuraSettings {
    effect_name: "krool_cannon_bullet2",
    bone: "body",
    position_x: 0.0,       // ← Edit X position offset here
    position_y: 0.0,       // ← Edit Y position offset here
    position_z: 0.0,       // ← Edit Z position offset here
    scale: 1.75,
    rotation_x: 0.0,       // ← Edit rotation here
    rotation_y: 0.0,      // ← Edit rotation here  
    rotation_z: 0.0,     // ← Edit rotation here
    color_r: 0.6,          // ← Edit color here (purple = 1.0, 0.5, 1.0)
    color_g: 0.35,          // ← Edit color here
    color_b: 0.7,          // ← Edit color here
    alpha: 1.0,            // ← Edit transparency here
    rate: 0.5,             // ← Edit animation speed here
};

// Keep existing universal effect tracker and other structs
#[derive(Debug, Clone)]
struct UniversalEffectTracker {
    handle: u32,
    spawn_frame: i32,
    target_visible_frame: f32,
    effect_hash: u64,
    last_status: i32,
    age_at_last_transition: i32,
    spawn_game_frame: i32,
    last_status_change_frame: i32,
    total_status_changes: u32,
    is_persistent: bool,
}

impl UniversalEffectTracker {
    fn new(handle: u32, spawn_frame: i32, target_frame: f32, effect_hash: u64, status: i32) -> Self {
        Self {
            handle,
            spawn_frame,
            target_visible_frame: target_frame,
            effect_hash,
            last_status: status,
            age_at_last_transition: 0,
            spawn_game_frame: spawn_frame,
            last_status_change_frame: spawn_frame,
            total_status_changes: 0,
            is_persistent: true,
        }
    }
    
    fn get_current_age(&self, current_frame: i32) -> i32 {
        current_frame - self.spawn_frame + self.age_at_last_transition
    }
    
    fn get_total_lifespan(&self, current_frame: i32) -> i32 {
        current_frame - self.spawn_game_frame
    }
    
    fn should_be_immediately_visible(&self, current_frame: i32) -> bool {
        self.get_current_age(current_frame) >= self.target_visible_frame as i32
    }
    
    fn update_for_status_transition(&mut self, current_frame: i32, new_status: i32) {
        self.age_at_last_transition = self.get_current_age(current_frame);
        self.last_status = new_status;
        self.spawn_frame = current_frame;
        self.last_status_change_frame = current_frame;
        self.total_status_changes += 1;
    }
}

#[derive(Debug, Copy, Clone)]
struct RolloutBombTracker {
    last_bomb_spawn_frame: i32,
    total_bombs_spawned: u32,
    was_hitting_last_frame: bool,
    last_enemy_damage_check: f32,
    consecutive_hit_frames: i32,
}

impl RolloutBombTracker {
    const fn new() -> Self {
        Self {
            last_bomb_spawn_frame: -100,
            total_bombs_spawned: 0,
            was_hitting_last_frame: false,
            last_enemy_damage_check: 0.0,
            consecutive_hit_frames: 0,
        }
    }
}

// Static tracking arrays for various effects across all players
static mut ROLLOUT_BOMB_TRACKERS: [RolloutBombTracker; 256] = [RolloutBombTracker::new(); 256];
static mut CUSTOM_AURA_HANDLE: [u32; 256] = [0; 256];
static mut LAST_FS_FLAG: [bool; 256] = [false; 256];
static mut LAST_MEGA_MODE: [bool; 256] = [false; 256];
static mut LAST_FS_STATUS: [i32; 256] = [-1; 256];
static mut LAST_CRY_SPAWN: [i32; 256] = [-60; 256];
static mut EVOLUTION_SPARKLE_HANDLE: [u32; 256] = [0; 256];
static mut EVOLUTION_SPARKLE_SPAWNED: [bool; 256] = [false; 256];
static mut DOWN_TAUNT_FRAME1_SPAWNED: [bool; 256] = [false; 256];
static mut DOWN_TAUNT_FRAME90_SPAWNED: [bool; 256] = [false; 256];
static mut LAST_DOWN_TAUNT_MOTION: [u64; 256] = [0; 256];
static mut CHARGE_MAX_FRAME1_SPAWNED: [bool; 256] = [false; 256];
static mut LAST_CHARGE_MAX_STATUS: [i32; 256] = [-1; 256];
static mut LAST_MAX_SIGN_FRAME: [i32; 256] = [-15; 256];
static mut LAST_SPEEDBOOSTER_FRAME: [i32; 256] = [-10; 256];
static mut LAST_STATUS_FOR_BOMB: [i32; 256] = [-1; 256];

static UNIVERSAL_EFFECTS: Lazy<Mutex<HashMap<String, UniversalEffectTracker>>> = 
    Lazy::new(|| Mutex::new(HashMap::new()));

// EFFECT CONFIGURATION for non-aura effects
struct EffectConfig {
    hash_str: &'static str,
    target_visible_frame: f32,
    scale: f32,
    bone_str: &'static str,
    rotation: (f32, f32, f32),
    alpha: f32,
    rate: f32,
}

// When using "EffectConfig::" new always use .with_params(#.#, #.#, (#.#, #.#, #.#));
impl EffectConfig {
    const fn new(hash_str: &'static str, visible_frame: f32, scale: f32, bone_str: &'static str) -> Self {
        Self {
            hash_str,
            target_visible_frame: visible_frame,
            scale,
            bone_str,
            rotation: (0.0, 0.0, 0.0),
            alpha: 1.0,
            rate: 1.0,
        }
    }
    
    const fn with_params(mut self, alpha: f32, rate: f32, rotation: (f32, f32, f32)) -> Self {
        self.alpha = alpha;
        self.rate = rate;
        self.rotation = rotation;
        self
    }
    
    fn get_hash(&self) -> Hash40 {
        Hash40::new(self.hash_str)
    }
    
    fn get_bone(&self) -> Hash40 {
        Hash40::new(self.bone_str)
    }
}



// GAS AURA PERSISTENT AURA SYSTEM
unsafe fn spawn_gastly_aura(
    boma: *mut BattleObjectModuleAccessor,
    fighter: &mut L2CFighterCommon
) -> u32 {
    let settings = GASTLY_AURA_SETTINGS;
    
    // Kill any existing aura first
    EffectModule::kill_kind(boma, Hash40::new(settings.effect_name), true, true);
    
    // ENHANCED: Use position offsets from settings
    let position_offset = Vector3f { 
        x: settings.position_x, 
        y: settings.position_y, 
        z: settings.position_z 
    };
    let rotation_vector = Vector3f { 
        x: settings.rotation_x, 
        y: settings.rotation_y, 
        z: settings.rotation_z 
    };
    
    // Spawn new aura with req_follow using position offsets
    let handle = EffectModule::req_follow(
        boma,
        Hash40::new(settings.effect_name),
        Hash40::new(settings.bone),
        &position_offset,      // Use position offset from settings
        &rotation_vector,      // Use rotation from settings
        settings.scale,
        true, // visibility
        0x40000, // category
        0,    // parent_id
        -1,   // unk
        0,    // level
        0,    // prev_handle
        false, // no_stop
        false  // unk2
    ) as u32; // Convert u64 to u32
    
    // Store handle in WorkModule
    WorkModule::set_int(boma, handle as i32, GASTLY_AURA_HANDLE_WORK_ID);
    
    // Apply visual modifications from consolidated settings
    
    // Apply RGB - use shiny colors for shiny slots (only for Purin)
    let is_shiny = unsafe { crate::is_shiny_gastly_costume(boma) };

    if is_shiny {
        EffectModule::set_rgb(boma, handle, 0.42, 0.75, 1.3); // Shiny blue aura
    } else {
        EffectModule::set_rgb(boma, handle, settings.color_r, settings.color_g, settings.color_b); // Normal purple aura
    }
    EffectModule::set_alpha(boma, handle, settings.alpha);
    EffectModule::set_rate(boma, handle, settings.rate);
    
    // Lock rotation to prevent bone influence
    EffectModule::set_rot(boma, handle, &rotation_vector);
    
    if false { // GASTLY_AURA_DEBUG_LOGGING
    }
    
    handle
}

// NEW: Add this function right after spawn_gastly_aura
pub unsafe fn spawn_gastly_aura_direct(boma: *mut BattleObjectModuleAccessor) -> u32 {
    // Use the same settings as your normal aura system
    let settings = crate::gastly::effects::GASTLY_AURA_SETTINGS;
    
    let position_offset = Vector3f { 
        x: settings.position_x, 
        y: settings.position_y, 
        z: settings.position_z 
    };
    let rotation_vector = Vector3f { 
        x: settings.rotation_x, 
        y: settings.rotation_y, 
        z: settings.rotation_z 
    };
    
    let handle = EffectModule::req_follow(
        boma,
        Hash40::new(settings.effect_name),
        Hash40::new(settings.bone),
        &position_offset,
        &rotation_vector,
        settings.scale,
        true, 0x40000, 0, -1, 0, 0, false, false
    ) as u32;
    
    if handle != u64::MAX as u32 && handle != 0 {
        // Apply visual modifications

        // Apply RGB - use shiny colors for shiny slots (only for Purin)
        let is_shiny = unsafe { crate::is_shiny_gastly_costume(boma) };

        if is_shiny {
            EffectModule::set_rgb(boma, handle, 0.42, 0.75, 1.3); // Shiny blue aura
        } else {
            EffectModule::set_rgb(boma, handle, settings.color_r, settings.color_g, settings.color_b); // Normal purple aura
        }
        EffectModule::set_alpha(boma, handle, settings.alpha);
        EffectModule::set_rate(boma, handle, settings.rate);
        EffectModule::set_rot(boma, handle, &rotation_vector);
        
        return handle;
    }
    
    0
}

pub unsafe fn spawn_persistent_gastly_aura_direct(boma: *mut BattleObjectModuleAccessor) -> u32 {
    let settings = crate::gastly::effects::GASTLY_AURA_SETTINGS;
    
    // Kill any existing aura effects first
    EffectModule::kill_kind(boma, Hash40::new(settings.effect_name), true, true);
    
    let position_offset = Vector3f { 
        x: settings.position_x, 
        y: settings.position_y, 
        z: settings.position_z 
    };
    let rotation_vector = Vector3f { 
        x: settings.rotation_x, 
        y: settings.rotation_y, 
        z: settings.rotation_z 
    };
    
    // Use req_follow with enhanced persistence settings
    let handle = EffectModule::req_follow(
        boma,
        Hash40::new(settings.effect_name),
        Hash40::new(settings.bone),
        &position_offset,
        &rotation_vector,
        settings.scale,
        true,    // visibility
        0x50000, // Enhanced category for persistence (changed from 0x40000)
        0,       // parent_id
        -1,      // unk
        0,       // level
        0,       // prev_handle
        false,   // no_stop
        false    // unk2
    ) as u32;
    
    if handle != u64::MAX as u32 && handle != 0 {
        // Apply visual modifications with extra persistence
        EffectModule::set_rgb(boma, handle, settings.color_r, settings.color_g, settings.color_b);
        EffectModule::set_alpha(boma, handle, settings.alpha);
        EffectModule::set_rate(boma, handle, settings.rate);
        EffectModule::set_rot(boma, handle, &rotation_vector);
        
        // Force visibility
        EffectModule::set_visible(boma, handle, true);
        
        return handle;
    }
    
    0
}


// NEW: Spawn aura on shadowball bone during shadowball mesh visibility

unsafe fn spawn_shadowball_bone_aura(
    boma: *mut BattleObjectModuleAccessor,
    fighter: &mut L2CFighterCommon,
    player_state: &PlayerEvolutionState  // ADD this parameter
) -> u32 {
    let settings = GASTLY_AURA_SETTINGS;
    
    // Kill any existing shadowball bone aura
    EffectModule::kill_kind(boma, Hash40::new(settings.effect_name), true, true);
    
    // Use position offsets from settings
    let position_offset = Vector3f { 
        x: settings.position_x, 
        y: settings.position_y, 
        z: settings.position_z 
    };
    let rotation_vector = Vector3f { 
        x: settings.rotation_x, 
        y: settings.rotation_y, 
        z: settings.rotation_z 
    };
    
    // Spawn on shadowball bone instead of body
    let handle = EffectModule::req_follow(
        boma,
        Hash40::new(settings.effect_name),
        Hash40::new("shadowball"),  // Different bone
        &position_offset,           // Use position offset
        &rotation_vector,           // Use rotation
        settings.scale * 0.8,       // Slightly smaller for shadowball
        true, 0x40000, 0, -1, 0, 0, false, false
    ) as u32;
    
    // Store in different work ID to avoid conflicts
    WorkModule::set_int(boma, handle as i32, GASTLY_AURA_HANDLE_WORK_ID + 1);
    
    // NEW: Apply evolution RGB immediately if we're evolving
    if player_state.is_evolving && 
       player_state.stage == EvolutionStage::Gastly && 
       player_state.evolution_target_stage == EvolutionStage::Haunter {
        // Evolution RGB
        EffectModule::set_rgb(boma, handle, 7.0, 7.0, 7.0);
    } else {
        // Normal RGB
        EffectModule::set_rgb(boma, handle, settings.color_r, settings.color_g, settings.color_b);
    }
    
    EffectModule::set_alpha(boma, handle, settings.alpha);
    EffectModule::set_rate(boma, handle, settings.rate);
    
    // Lock rotation
    EffectModule::set_rot(boma, handle, &rotation_vector);
    
    handle
}

// NEW: Spawn aura on shadowball bone during shadowball mesh visibility
unsafe fn should_suppress_gastly_aura(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState
) -> bool {
    let current_status = StatusModule::status_kind(boma);
    let current_motion = MotionModule::motion_kind(boma);
    
    // Check for results screen motions
    let is_results_motion = current_motion == 0x7fb997a80 || // First results motion
                           current_motion == 0x42af5a458;   // Second results motion
    
    // NEVER suppress during these special statuses or motions
    if current_status == *FIGHTER_STATUS_KIND_REBIRTH ||      // Rebirth platform
       current_status == *FIGHTER_STATUS_KIND_WIN ||         // Win pose (0x1DA)
       current_status == *FIGHTER_STATUS_KIND_LOSE ||        // Lose pose (0x1DB)
       current_status == *FIGHTER_STATUS_KIND_ENTRY ||       // Entry animation (0x1D9)
       is_results_motion ||                                   // Results screen motions
       (current_status >= 0x190 && current_status <= 0x1A0) { // Results range
        return false;
    }
    
    // Use your original suppression logic for everything else
    let current_motion_hash_val = MotionModule::motion_kind(boma);
    let motion_hash = Hash40 { hash: current_motion_hash_val };
    
    if motion_hash.hash == smash::hash40("squat_wait") ||
       motion_hash.hash == smash::hash40("swim_rise") ||
       motion_hash.hash == smash::hash40("swim_up") ||
       motion_hash.hash == smash::hash40("swim_up_damage") ||
       motion_hash.hash == smash::hash40("swim") ||
       motion_hash.hash == smash::hash40("swim_f") || 
       motion_hash.hash == smash::hash40("swim_b") || 
       motion_hash.hash == smash::hash40("swim_end") || 
       motion_hash.hash == smash::hash40("swim_turn") || 
       motion_hash.hash == smash::hash40("swim_drown") || 
       motion_hash.hash == smash::hash40("swim_drown_out") {
        return true;
    }
    
    let is_rollout = current_status == PURIN_SPECIAL_N_ROLL || 
                     current_status == PURIN_SPECIAL_N_ROLL_AIR;
    
    if is_rollout {
        let has_active_hitbox = AttackModule::is_attack(boma, 0, false) ||
                               AttackModule::is_attack(boma, 1, false) ||
                               AttackModule::is_attack(boma, 2, false) ||
                               AttackModule::is_infliction_status(boma, 0) ||
                               AttackModule::is_attack_occur(boma);
        
        if has_active_hitbox {
            return true;
        }
    }
    
    false
}

// Handle Gastly gas persistent aura
unsafe fn handle_gastly_aura(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon
) {
    //  Only show aura for Gastly stage
    if player_state.stage != EvolutionStage::Gastly {
        cleanup_gastly_aura(boma);
        
        // Also clean up shadowball bone aura
        let shadowball_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID + 1) as u32;
        if shadowball_handle != 0 {
            EffectModule::kill(boma, shadowball_handle, false, true);
            WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID + 1);
        }
        return;
    }

    let current_status = StatusModule::status_kind(boma);
    let force_spawn_aura = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
    
    if should_suppress_gastly_aura(boma, player_state) {
        let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
        if stored_handle != 0 && EffectModule::is_exist_effect(boma, stored_handle) {
            EffectModule::set_visible(boma, stored_handle, false);
        }
    } else {
        let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
        
        if stored_handle == 0 || !EffectModule::is_exist_effect(boma, stored_handle) || force_spawn_aura {
            if force_spawn_aura {
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
            }
            spawn_gastly_aura(boma, fighter);
        } else {
            EffectModule::set_visible(boma, stored_handle, true);
            check_and_apply_evolution_aura_rgb(boma, player_state, stored_handle);
        }
    }
}

// Handle evolution Gastly aura RGB changes
unsafe fn check_and_apply_evolution_aura_rgb(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    aura_handle: u32
) {
    // Only apply during Gastly → Haunter evolution
    if player_state.is_evolving && 
       player_state.stage == EvolutionStage::Gastly && 
       player_state.evolution_target_stage == EvolutionStage::Haunter {
        
        // Set bright white RGB during Gastly → Haunter evolution
        EffectModule::set_rgb(boma, aura_handle, 7.0, 7.0, 7.0);
        
        // Keep original alpha and rate
        let settings = GASTLY_AURA_SETTINGS;
        EffectModule::set_alpha(boma, aura_handle, settings.alpha);
        EffectModule::set_rate(boma, aura_handle, settings.rate);
        
        // Debug logging
        if player_state.evolution_timer % 60 == 0 {
        }
        
    } else {
        // Normal RGB when not evolving or different evolution
        let settings = GASTLY_AURA_SETTINGS;

        // Apply RGB - use shiny colors for shiny slots (only for Purin)
        let is_shiny = unsafe { crate::is_shiny_gastly_costume(boma) };

        if is_shiny {
            EffectModule::set_rgb(boma, aura_handle, 0.42, 0.75, 1.3); // Shiny blue aura
        } else {
            EffectModule::set_rgb(boma, aura_handle, settings.color_r, settings.color_g, settings.color_b); // Normal purple aura
        }
        EffectModule::set_alpha(boma, aura_handle, settings.alpha);
        EffectModule::set_rate(boma, aura_handle, settings.rate);
    }
}

pub unsafe fn handle_gastly_aura_with_rebirth_detection(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon
) {
    let current_status = StatusModule::status_kind(boma);
    let force_spawn_aura = WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
    let is_rebirth = current_status == *FIGHTER_STATUS_KIND_REBIRTH;
    
    // Use updated suppression function that doesn't suppress during rebirth
    if should_suppress_gastly_aura(boma, player_state) {
        // Hide aura during excluded states (but NOT during rebirth)
        let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
        if stored_handle != 0 && EffectModule::is_exist_effect(boma, stored_handle) {
            EffectModule::set_visible(boma, stored_handle, false);
        }
    } else {
        // Ensure aura is active and visible (including during rebirth)
        let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
        
        if stored_handle == 0 || !EffectModule::is_exist_effect(boma, stored_handle) || force_spawn_aura {
            // Spawn new aura
            if force_spawn_aura {
                // Clear the flag after using it
                WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
            }
            spawn_gastly_aura(boma, fighter);
        } else {
            // Make sure existing aura is visible
            EffectModule::set_visible(boma, stored_handle, true);
        }
    }
}

// UNIVERSAL EFFECT SPAWNING FUNCTION for non-aura effects (unchanged)
unsafe fn spawn_universal_effect(
    fighter: &mut L2CFighterCommon,
    config: &EffectConfig,
    key: &str,
    current_frame: i32,
    current_status: i32,
    force_immediate_visibility: bool
) -> Option<u32> {
    let effect_hash = config.get_hash();
    let bone_hash = config.get_bone();
    
    macros::EFFECT_FLW_POS(
        fighter,
        effect_hash,
        bone_hash,
        0.0, 0.0, 0.0,
        config.rotation.0, config.rotation.1, config.rotation.2,
        config.scale,
        true
    );
    EffectModule::enable_sync_init_pos_last(fighter.module_accessor);
    macros::LAST_EFFECT_SET_ALPHA(fighter, config.alpha);
    macros::LAST_EFFECT_SET_RATE(fighter, config.rate);
    
    let new_handle = EffectModule::get_last_handle(fighter.module_accessor) as u32;
    
    if force_immediate_visibility || current_frame > 0 {
        EffectModule::set_frame(fighter.module_accessor, new_handle, config.target_visible_frame);
    }
    
    // Store in universal tracker
    if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
        let tracker = UniversalEffectTracker::new(
            new_handle, 
            current_frame, 
            config.target_visible_frame, 
            effect_hash.hash, 
            current_status
        );
        effects.insert(key.to_string(), tracker);
    }
    
    Some(new_handle)
}

// UNIVERSAL EFFECT PERSISTENCE CHECK (unchanged)
unsafe fn should_preserve_effect(
    boma: *mut BattleObjectModuleAccessor,
    tracker: &UniversalEffectTracker,
    _current_status: i32,
    current_frame: i32
) -> bool {
    if !EffectModule::is_exist_effect(boma, tracker.handle) {
        return false;
    }
    
    if tracker.should_be_immediately_visible(current_frame) {
        return true;
    }
    
    let effect_age = tracker.get_current_age(current_frame);
    if effect_age < 5 {
        return true;
    }
    
    false
}

// UNIVERSAL EFFECT MANAGEMENT FUNCTION (unchanged)
unsafe fn manage_universal_effect(
    fighter: &mut L2CFighterCommon,
    boma: *mut BattleObjectModuleAccessor,
    config: &EffectConfig,
    key: &str,
    current_frame: i32,
    current_status: i32,
    condition_met: bool,
    force_immediate: bool
) {
    let mut need_new_effect = false;
    let mut preserve_age = 0;
    
    if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
        if let Some(tracker) = effects.get_mut(key) {
            if condition_met {
                if should_preserve_effect(boma, tracker, current_status, current_frame) {
                    if tracker.last_status != current_status {
                        preserve_age = tracker.get_current_age(current_frame);
                        tracker.update_for_status_transition(current_frame, current_status);
                    }
                    return;
                } else {
                    EffectModule::kill(boma, tracker.handle, false, true);
                    need_new_effect = true;
                    preserve_age = tracker.get_current_age(current_frame);
                }
            } else {
                EffectModule::kill(boma, tracker.handle, false, true);
                effects.remove(key);
                return;
            }
        } else if condition_met {
            need_new_effect = true;
        }
    }
    
    if need_new_effect && condition_met {
        let force_immediate_local = force_immediate || preserve_age >= config.target_visible_frame as i32;
        
        spawn_universal_effect(
            fighter, 
            config, 
            key, 
            current_frame, 
            current_status, 
            force_immediate_local
        );
    }
}

// MAIN EFFECT HANDLER - UPDATED TO USE GASTLY GAS AURA SYSTEM
pub unsafe fn handle_gastly_effects(
    boma: *mut BattleObjectModuleAccessor, 
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon
) {

    // Only apply to marked Gastly costumes (Purin only)
    if !crate::is_marked_gastly_costume(boma) {
        return;
    }

    let current_status = StatusModule::status_kind(boma);
    let current_frame = player_state.current_frame;
    
    // Skip effect management during death/rebirth to prevent conflicts with cleanup
    let is_dead_or_rebirth = current_status == *smash::lib::lua_const::FIGHTER_STATUS_KIND_DEAD || 
                            current_status == *smash::lib::lua_const::FIGHTER_STATUS_KIND_REBIRTH;
    
    // Also check for recent death to prevent effects spam in duo matches
    static mut RECENT_DEATH_STATUS: [bool; 256] = [false; 256];
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    
    if instance_key < 256 {
        if is_dead_or_rebirth {
            RECENT_DEATH_STATUS[instance_key] = true;
            return;
        }
        
        // If we recently died, skip effects for a few frames to prevent spam
        if RECENT_DEATH_STATUS[instance_key] {
            // Check if we're in a stable state (not in transition statuses)
            let stable_statuses = [
                *smash::lib::lua_const::FIGHTER_STATUS_KIND_WAIT,
                *smash::lib::lua_const::FIGHTER_STATUS_KIND_WALK,
                *smash::lib::lua_const::FIGHTER_STATUS_KIND_DASH,
                *smash::lib::lua_const::FIGHTER_STATUS_KIND_RUN,
            ];
            if stable_statuses.contains(&current_status) {
                RECENT_DEATH_STATUS[instance_key] = false; // Clear death flag
            } else {
                return; // Still in transition, skip effects
            }
        }
    }
    
    // SMASH BALL AURA REPLACEMENT - HANDLE FIRST
    let is_final_smash_flag = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);

    // NEW: Handle Final Smash special effects
    handle_final_smash_special_effects(boma, player_state, fighter, current_status, current_frame, is_final_smash_flag);

    // Kill custom aura when final smash actually starts (status 0x1E0)
    if current_status == 0x1E0 {  // FINAL status
        // Try multiple methods to kill the custom aura
        macros::EFFECT_OFF_KIND(fighter, Hash40::new("sys_special_all_up"), false, false);
        EffectModule::kill_kind(boma, Hash40::new("sys_special_all_up"), false, true);
        if instance_key < 256 && CUSTOM_AURA_HANDLE[instance_key] != 0 {
            EffectModule::kill(boma, CUSTOM_AURA_HANDLE[instance_key], false, true);
            CUSTOM_AURA_HANDLE[instance_key] = 0;
        }
    }
    // Check if we have smash ball + are Gengar + have mega/giga mode set (but NOT in final smash)
    else if is_final_smash_flag && 
       current_status != 0x1E0 &&  // NOT in FINAL status
       current_status != 0x1E8 &&  // NOT in FINAL_WAIT status  
       current_status != 0x1E9 &&  // NOT in FINAL_END status
       player_state.stage == EvolutionStage::Gengar &&
       (player_state.mega_gengar_form_active || player_state.giga_gengar_form_active) {
        
        // Kill default final smash aura effects
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura_charge"), false, true);
        EffectModule::kill_kind(boma, Hash40::new("sys_final_aura2"), false, true);
        
        // Check if we need to spawn our custom purple aura
        if instance_key < 256 {
            // Check if our custom aura exists
            if CUSTOM_AURA_HANDLE[instance_key] == 0 || 
               !EffectModule::is_exist_effect(boma, CUSTOM_AURA_HANDLE[instance_key]) {
                
                // Spawn new custom purple aura
                let handle = EffectModule::req_follow(
                    boma,
                    Hash40::new("sys_special_all_up"),
                    Hash40::new("body"),
                    &Vector3f { x: 0.0, y: 0.0, z: 0.0 },
                    &Vector3f { x: 0.0, y: 90.0, z: 0.0 },
                    1.0, true, 0x40000, 0, -1, 0, 0, false, false
                ) as u32;
                
                if handle != u64::MAX as u32 && handle != 0 {
                    EffectModule::set_rgb(boma, handle, 1.0, 0.2, 1.0); // Purple
                    EffectModule::set_alpha(boma, handle, 1.0);
                    CUSTOM_AURA_HANDLE[instance_key] = handle;
                }
            }
        }
    } else {
        // Clean up custom aura if conditions not met
        if instance_key < 256 && CUSTOM_AURA_HANDLE[instance_key] != 0 {
            EffectModule::kill(boma, CUSTOM_AURA_HANDLE[instance_key], false, true);
            CUSTOM_AURA_HANDLE[instance_key] = 0;
        }
    }
    
    // Handle evolution flash effect
    handle_evolution_effects(boma, player_state, fighter, current_status, current_frame);

    // Handle Gastly persistent aura - ONLY for Gastly stage
    if player_state.stage == EvolutionStage::Gastly {
        // Use the SAME shadowball detection logic that worked before
        let is_shadowball_hold = current_status == PURIN_SPECIAL_N_HOLD || 
                                current_status == PURIN_SPECIAL_N_HOLD_MAX;
        
        let shadowball_mesh_should_be_visible = if is_shadowball_hold {
            let current_motion_hash_val = MotionModule::motion_kind(boma);
            let is_air_motion = current_motion_hash_val == SPECIAL_AIR_N_HOLD_MOTION.hash ||
                            current_motion_hash_val == SPECIAL_AIR_N_HOLD_MAX_MOTION.hash;
            
            let frame_threshold = if is_air_motion {
                SHADOWBALL_MESH_DELAY_FRAMES_AIR
            } else {
                SHADOWBALL_MESH_DELAY_FRAMES_GROUND
            };
            
            let official_charge_count = WorkModule::get_float(boma, PURIN_WORK_FLOAT_CHARGE_COUNT);
            let is_max_charge = WorkModule::is_flag(boma, PURIN_FLAG_MAX_FLAG);
            let hold_frames = WorkModule::get_int(boma, PURIN_WORK_INT_HOLD);
            
            player_state.shadowball_status_frames > frame_threshold || 
            official_charge_count > frame_threshold as f32 || 
            hold_frames > frame_threshold ||
            is_max_charge
        } else {
            false
        };

        if shadowball_mesh_should_be_visible {
            // Use shadowball bone (this worked in your old code!)
            cleanup_gastly_aura(boma);
            
            let shadowball_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID + 1) as u32;
            if shadowball_handle == 0 || !EffectModule::is_exist_effect(boma, shadowball_handle) {
                spawn_shadowball_bone_aura(boma, fighter, player_state);
            } else {
                // NEW: Apply evolution RGB to shadowball aura too
                check_and_apply_evolution_aura_rgb(boma, player_state, shadowball_handle);
            }
        } else {
            // Use regular body aura
            let shadowball_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID + 1) as u32;
            if shadowball_handle != 0 {
                EffectModule::kill(boma, shadowball_handle, false, true);
                WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID + 1);
            }
            
            handle_gastly_aura(boma, player_state, fighter);
        }
    } else {
        // Clean up ALL aura effects when not Gastly
        cleanup_gastly_aura(boma);
        let shadowball_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID + 1) as u32;
        if shadowball_handle != 0 {
            EffectModule::kill(boma, shadowball_handle, false, true);
            WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID + 1);
        }
    }
        
    // NEW: Handle down taunt effects for all pokemon stages
    handle_down_taunt_effects(boma, player_state, current_status, current_frame);
    
    // NEW: Handle special n charge max effects
    handle_special_n_charge_max_effects(boma, player_state, current_status, current_frame);
    
    // Handle Mewtwo shadowball effects
    handle_shadowball_effects(boma, player_state, fighter, current_status, current_frame);
}

// NEW: Handle Final Smash special effects
unsafe fn handle_final_smash_special_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    current_status: i32,
    current_frame: i32,
    is_final_smash_flag: bool
) {
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }

    // Track previous states for trigger detection
    
    let fs_flag_just_gained = !LAST_FS_FLAG[instance_key] && is_final_smash_flag;
    let mega_mode_just_activated = !LAST_MEGA_MODE[instance_key] && player_state.mega_gengar_form_active;
    let status_just_changed = LAST_FS_STATUS[instance_key] != current_status;
    
    // Lucario Mega Symbol - when BOTH smash ball + mega mode are active
    if player_state.stage == EvolutionStage::Gengar && 
       is_final_smash_flag && 
       player_state.mega_gengar_form_active {
        
        // Trigger symbol when either condition becomes true (while the other is already true)
        let should_spawn_symbol = fs_flag_just_gained ||  // Got smash ball while mega mode active
                                 mega_mode_just_activated; // Activated mega mode while having smash ball
        
        if should_spawn_symbol {
            let position_offset = Vector3f { x: 0.0, y: 25.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("lucario_final_megasymbol"),
                Hash40::new("top"),
                &position_offset,
                &rotation_vector,
                1.0,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                EffectModule::set_rate(boma, handle, 0.7);
            }
        }
    }
    
    // Handle final smash status effects only if we have the flag
    if is_final_smash_flag {
        match current_status {
            0x1E0 => { // FINAL status
            },
            
            0x1E8 => { // FINALWAIT status
                // Bayonetta Final Cry - During entire finalwait status
                spawn_bayonetta_final_cry(boma, current_frame);
                
            },
            
            0x1E9 => { // FINALEND status
                // Kill Bayonetta Final Cry when exiting finalwait
                if status_just_changed {
                    macros::EFFECT_OFF_KIND(fighter, Hash40::new("bayonetta_final_cry"), false, false);
                    EffectModule::kill_kind(boma, Hash40::new("bayonetta_final_cry"), false, true);
                }
                                
                // Koopa Final Disappear - only for Mega/Giga Gengar forms
                if status_just_changed && 
                   player_state.stage == EvolutionStage::Gengar &&
                   (player_state.mega_gengar_form_active || player_state.giga_gengar_form_active) {
                    
                    let position_offset = Vector3f { x: 0.0, y: 6.0, z: 0.0 };
                    let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
                    
                    let handle = EffectModule::req_follow(
                        boma,
                        Hash40::new("koopa_final_disappear"),
                        Hash40::new("top"),
                        &position_offset,
                        &rotation_vector,
                        0.5, // scale: 0.5
                        true, 0x40000, 0, -1, 0, 0, false, false
                    ) as u32;
                    
                    if handle != u64::MAX as u32 && handle != 0 {
                    }
                }
            },
            
            _ => {
                // Clean up cry effect if we exit final smash statuses unexpectedly
                if !is_final_smash_status(current_status) {
                    macros::EFFECT_OFF_KIND(fighter, Hash40::new("bayonetta_final_cry"), false, false);
                    EffectModule::kill_kind(boma, Hash40::new("bayonetta_final_cry"), false, true);
                }
            }
        }
    }
    // Update tracking variables
    LAST_FS_FLAG[instance_key] = is_final_smash_flag;
    LAST_MEGA_MODE[instance_key] = player_state.mega_gengar_form_active;
    LAST_FS_STATUS[instance_key] = current_status;
}

// Helper function to check if we're in any final smash status
unsafe fn is_final_smash_status(status: i32) -> bool {
    status == 0x1E0 || status == 0x1E8 || status == 0x1E9
}

// Spawn Bayonetta Final Cry effect (during finalwait)
unsafe fn spawn_bayonetta_final_cry(boma: *mut BattleObjectModuleAccessor, current_frame: i32) {
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    
    if instance_key < 256 && (current_frame - LAST_CRY_SPAWN[instance_key] >= 60) {
        let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
        
        let handle = EffectModule::req_follow(
            boma,
            Hash40::new("bayonetta_final_cry"),
            Hash40::new("body"),
            &position_offset,
            &rotation_vector,
            1.0,
            true, 0x40000, 0, -1, 0, 0, false, false
        ) as u32;
        
        if handle != u64::MAX as u32 && handle != 0 {
            LAST_CRY_SPAWN[instance_key] = current_frame;
        }
    }
}

// EVOLUTION EFFECTS (unchanged from previous version)
unsafe fn handle_evolution_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    _current_status: i32,
    current_frame: i32
) {
    // Static tracking for evolution sparkle effect
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    
    if player_state.is_evolving {
        // Spawn sys_fairybottle_navy2 effect once at start of evolution
        if instance_key < 256 && !EVOLUTION_SPARKLE_SPAWNED[instance_key] {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("sys_fairybottle_navy2"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                5.0, // scale: 5.0 for visibility
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                EffectModule::set_rgb(boma, handle, 1.0, 5.0, 1.0);
                EffectModule::set_alpha(boma, handle, 1.0);
                EffectModule::set_rate(boma, handle, 0.5);
                EVOLUTION_SPARKLE_HANDLE[instance_key] = handle;
                EVOLUTION_SPARKLE_SPAWNED[instance_key] = true;
            }
        }
        
        
    } else {
        // Clean up evolution sparkle effect when evolution ends
        if instance_key < 256 && EVOLUTION_SPARKLE_SPAWNED[instance_key] {
            if EVOLUTION_SPARKLE_HANDLE[instance_key] != 0 && 
               EffectModule::is_exist_effect(boma, EVOLUTION_SPARKLE_HANDLE[instance_key]) {
                EffectModule::kill(boma, EVOLUTION_SPARKLE_HANDLE[instance_key], false, true);
            }
            EVOLUTION_SPARKLE_HANDLE[instance_key] = 0;
            EVOLUTION_SPARKLE_SPAWNED[instance_key] = false;
        }
        
        // Ensure normal color state (no lingering flash effects)
        macros::COL_NORMAL(fighter);
        
        // Handle completion effects - UPDATED TO USE req_follow
        if player_state.evolution_just_completed_this_frame {
            let position_offset = Vector3f { x: 0.0, y: 6.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("lucario_final_end"),
                Hash40::new("top"),
                &position_offset,
                &rotation_vector,
                0.7,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
            }
            
            player_state.evolution_just_completed_this_frame = false;

            // Trigger shiny effect for post-evolution stages
            if player_state.is_shiny && (player_state.stage == EvolutionStage::Haunter || player_state.stage == EvolutionStage::Gengar) {
                player_state.shiny_effect_pending = true;
                player_state.shiny_effect_delay_timer = 75; // 75 frames delay
                player_state.evolution_completion_frame = player_state.current_frame;
            }

            player_state.frames_since_level_up_effect = 0;
        }

        // Level up effect sequence (same as before)
        if player_state.frames_since_level_up_effect == 20 {
            let position_offset = Vector3f { x: 0.0, y: 6.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("sys_level_up"),
                Hash40::new("top"),
                &position_offset,
                &rotation_vector,
                0.5,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
            }
        }

        if player_state.frames_since_level_up_effect == 30 {
            let position_offset = Vector3f { x: 0.0, y: 6.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("sys_pokemon_out"),
                Hash40::new("top"),
                &position_offset,
                &rotation_vector,
                1.5,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
            }
        }

        if player_state.frames_since_level_up_effect == 55 {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("edge_light_impact"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                1.0,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                EffectModule::set_rate(boma, handle, 0.3);
            }
        }

        // Evolution cry sounds 15 frames after last effect (frame 70)
        if player_state.frames_since_level_up_effect == 70 {
            let cry_sound = match player_state.stage {
                EvolutionStage::Haunter => "cry_haunter",  // Just evolved from Gastly
                EvolutionStage::Gengar => "cry_gengar",    // Just evolved from Haunter
                _ => "", // Gastly doesn't have evolution completion (starts as Gastly)
            };
            
            if !cry_sound.is_empty() {
            // Check if shiny effect will delay the cry
            let cry_delay = if player_state.is_shiny && 
                            (player_state.stage == EvolutionStage::Haunter || player_state.stage == EvolutionStage::Gengar) {
                60 // Delay cry by 28 frames for shiny (103 - 43 = 60)
            } else {
                0 // No delay for non-shiny
            };
            
            if cry_delay > 0 {
                // Schedule delayed cry sound
                player_state.delayed_cry_sound = cry_sound.to_string();
                player_state.delayed_cry_timer = cry_delay;
            } else {
                // Play cry immediately
                let cry_handle = SoundModule::play_se(
                    boma,
                    Hash40::new(cry_sound),
                    false, false, false, false,
                    smash::app::enSEType(0)
                );
                SoundModule::set_se_vol(boma, cry_handle as i32, 2.5, 0);
                
                // Track cry sound playback (existing code)
                let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
                match player_state.stage {
                    EvolutionStage::Haunter => {
                        crate::gastly::ui_management::track_cry_sound_playback(
                            Hash40::new("cry_haunter"),
                            player_state.current_frame,
                            entry_id
                        );
                    },
                    EvolutionStage::Gengar => {
                        crate::gastly::ui_management::track_cry_sound_playback(
                            Hash40::new("cry_gengar"),
                            player_state.current_frame,
                            entry_id
                        );
                    },
                    _ => {}
                }
                
                // Set flag to trigger cutin on next UI update (if enabled)
                if crate::gastly::constants::ENABLE_EVOLUTION_CUTINS {
                    match player_state.stage {
                        EvolutionStage::Haunter => {
                            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
                        },
                        EvolutionStage::Gengar => {
                            WorkModule::set_flag(boma, true, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
                        },
                        _ => {}
                    }
                }

            }
        }
            
            player_state.frames_since_level_up_effect = -1;
        }

        if player_state.frames_since_level_up_effect >= 0 {
            player_state.frames_since_level_up_effect += 1;
        }
        
        // Handle evolution cancellation effects
        if player_state.evolution_just_cancelled_this_frame {
            let position_offset = Vector3f { x: 0.0, y: 6.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("sys_deku_flash"),
                Hash40::new("top"),
                &position_offset,
                &rotation_vector,
                0.3,
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                EffectModule::set_rate(boma, handle, 0.5);
                EffectModule::set_rgb(boma, handle, 0.3, 0.0, 0.0);
                EffectModule::set_alpha(boma, handle, 1.5);
            }
            
            player_state.evolution_cancel_fade_timer = 0;
            player_state.evolution_just_cancelled_this_frame = false;
        }

        // Handle cancellation fade timer (brief red flash for cancellation feedback)
        if player_state.evolution_cancel_fade_timer >= 0 {
            player_state.evolution_cancel_fade_timer += 1;
            
            if player_state.evolution_cancel_fade_timer <= 45 {
                let fade_progress = player_state.evolution_cancel_fade_timer as f32 / 45.0;
                let current_alpha = 0.5 * (1.0 - fade_progress);
                
                macros::FLASH(fighter, 0.6, 0.0, 0.0, current_alpha);
            } else {
                player_state.evolution_cancel_fade_timer = -1;
                macros::COL_NORMAL(fighter);
            }
        }
    }
}

unsafe fn handle_down_taunt_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    current_status: i32,
    current_frame: i32
) {
    // Static tracking for down taunt effects
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    // Check if we're in down taunt by motion instead of status
    let current_motion = MotionModule::motion_kind(boma);
    let is_appeal_lw_l = current_motion == smash::hash40("appeal_lw_l");
    let is_appeal_lw_r = current_motion == smash::hash40("appeal_lw_r");
    let is_down_taunt = is_appeal_lw_l || is_appeal_lw_r;
    
    if is_down_taunt {
        let motion_frame = MotionModule::frame(boma) as i32;
        
        // Reset spawn flags when entering new down taunt motion
        if LAST_DOWN_TAUNT_MOTION[instance_key] != current_motion {
            DOWN_TAUNT_FRAME1_SPAWNED[instance_key] = false;
            DOWN_TAUNT_FRAME90_SPAWNED[instance_key] = false;
            LAST_DOWN_TAUNT_MOTION[instance_key] = current_motion;
        }
        
        // Frame 1 effect (one-shot)
        if motion_frame >= 1 && !DOWN_TAUNT_FRAME1_SPAWNED[instance_key] {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("bayonetta_batwithin_change"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                1.0, // Default scale
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                DOWN_TAUNT_FRAME1_SPAWNED[instance_key] = true;
            }
        }
        
        // Frame 90 effect (one-shot) - CHANGED FROM 100 TO 90
        if motion_frame >= 90 && !DOWN_TAUNT_FRAME90_SPAWNED[instance_key] {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("bayonetta_batwithin_change"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                1.0, // Default scale
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                DOWN_TAUNT_FRAME90_SPAWNED[instance_key] = true;
            }
        }
    } else {
        // Reset flags when not in down taunt motion
        if LAST_DOWN_TAUNT_MOTION[instance_key] != 0 {
            DOWN_TAUNT_FRAME1_SPAWNED[instance_key] = false;
            DOWN_TAUNT_FRAME90_SPAWNED[instance_key] = false;
            LAST_DOWN_TAUNT_MOTION[instance_key] = 0;
        }
    }
}

// NEW: Handle special n charge max effects
unsafe fn handle_special_n_charge_max_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    current_status: i32,
    current_frame: i32
) {
    // Static tracking for charge max effects
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    // Check if we're in special n charge max status
    let is_charge_max = current_status == PURIN_SPECIAL_N_HOLD_MAX; // 0x1E2
    
    if is_charge_max {
        // Reset spawn flag when entering new charge max status
        if LAST_CHARGE_MAX_STATUS[instance_key] != current_status {
            CHARGE_MAX_FRAME1_SPAWNED[instance_key] = false;
            LAST_CHARGE_MAX_STATUS[instance_key] = current_status;
        }
        
        // Frame 1 effect (one-shot)
        if !CHARGE_MAX_FRAME1_SPAWNED[instance_key] {
            let position_offset = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            let rotation_vector = Vector3f { x: 0.0, y: 90.0, z: 0.0 };
            
            let handle = EffectModule::req_follow(
                boma,
                Hash40::new("chrom_final_light2"),
                Hash40::new("body"),
                &position_offset,
                &rotation_vector,
                0.7, // Scale: 0.7 as requested
                true, 0x40000, 0, -1, 0, 0, false, false
            ) as u32;
            
            if handle != u64::MAX as u32 && handle != 0 {
                CHARGE_MAX_FRAME1_SPAWNED[instance_key] = true;
            }
        }
    } else {
        // Reset flag when not in charge max status
        if LAST_CHARGE_MAX_STATUS[instance_key] != -1 {
            CHARGE_MAX_FRAME1_SPAWNED[instance_key] = false;
            LAST_CHARGE_MAX_STATUS[instance_key] = -1;
        }
    }
}

// MEWTWO SHADOWBALL EFFECTS
unsafe fn handle_shadowball_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &mut PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    current_status: i32,
    current_frame: i32
) {
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    let entry_id = (instance_key / 32) as usize; // For backward compatibility with UI effects

    // Always kill purin_appeal_lw effect
    EffectModule::kill_kind(boma, Hash40::new("purin_appeal_lw"), false, false);
    
    let shadowball_state = detect_shadowball_hitbox_state(boma, player_state);
    
    // Enhanced shadowball effect logic based on hitbox states
    match shadowball_state {
        ShadowballState::ChargedRolloutWithHitbox | 
        ShadowballState::RegularRolloutWithHitbox | 
        ShadowballState::AirToGroundRolloutWithHitbox => {
            // Main shadowball effect (make key unique per player)
            let shadowball_config = EffectConfig::new("mewtwo_shadowball", 15.0, 2.0, "body")
                .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
            
            let shadowball_key = format!("mewtwo_shadowball_main_{}", instance_key);
            manage_universal_effect(
                fighter, boma, &shadowball_config, &shadowball_key,
                current_frame, current_status, true, true
            );
            
            // Tail effect (make key unique per player)
            let tail_config = EffectConfig::new("mewtwo_shadowball_tail", 0.0, 2.0, "body")
                .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
            
            let tail_key = format!("mewtwo_shadowball_tail_{}", instance_key);
            manage_universal_effect(
                fighter, boma, &tail_config, &tail_key,
                current_frame, current_status, true, true
            );
            
            // Suppress vanilla Purin effects during invisible rollout
            let purin_effects = [
                "purin_appeal_lw", "purin_attack_arc_d", "purin_final_bg_black",
                "purin_final_bg_vortex", "purin_final_shockwave", "purin_final_smoke",
                "purin_hataku", "purin_hataku_flash", "purin_hataku_hold",
                "purin_korogaru", "purin_korogaru2", "purin_korogaru_loop",
                "purin_korogaru_max", "purin_korogaru_wind", "purin_nemuru",
                "purin_nemuru_end", "purin_nemuru_start", "purin_sleep",
                "purin_smash_arc", "purin_smash_line", "purin_utau",
                "sys_dash_smoke", "sys_flash"
            ];
            
            for effect_name in purin_effects.iter() {
                EffectModule::kill_kind(boma, Hash40::new(effect_name), false, false);
            }
        },
        
        // During rollout WITHOUT active hitboxes - clean up shadowball effects
        ShadowballState::ChargedRollout |
        ShadowballState::RegularRollout |
        ShadowballState::AirToGroundRollout => {
            // Clean up shadowball effects when no hitboxes are active (per-player cleanup)
            if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
                let shadowball_keys: Vec<String> = effects.keys()
                    .filter(|k| k.contains("mewtwo_shadowball") && k.ends_with(&format!("_{}", instance_key)))
                    .cloned()
                    .collect();
                
                for key in shadowball_keys {
                    if let Some(tracker) = effects.remove(&key) {
                        EffectModule::kill(boma, tracker.handle, false, true);
                    }
                }
            }
        },
        
        // Hold statuses with shadowball mesh - existing logic
        ShadowballState::ActiveFrameBased | ShadowballState::ActiveWithHitbox => {
            let is_hold_status = current_status == 0x1E1 || current_status == 0x1E2;
            
            if is_hold_status {
                // Main shadowball effect (unique per player)
                let shadowball_config = EffectConfig::new("mewtwo_shadowball", 15.0, 2.0, "body")
                    .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
                
                let shadowball_key = format!("mewtwo_shadowball_main_{}", instance_key);
                manage_universal_effect(
                    fighter, boma, &shadowball_config, &shadowball_key,
                    current_frame, current_status, true, true
                );
                
                // Tail effect (unique per player)
                let tail_config = EffectConfig::new("mewtwo_shadowball_tail", 0.0, 2.0, "body")
                    .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
                
                let tail_key = format!("mewtwo_shadowball_tail_{}", instance_key);
                manage_universal_effect(
                    fighter, boma, &tail_config, &tail_key,
                    current_frame, current_status, true, true
                );
                
                // Hold-specific effect (unique per player)
                let hold_config = EffectConfig::new("mewtwo_shadowball_hold", 15.0, 2.0, "body")
                    .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
                
                let hold_key = format!("mewtwo_shadowball_hold_{}", instance_key);
                manage_universal_effect(
                    fighter, boma, &hold_config, &hold_key,
                    current_frame, current_status, true, false
                );
                
                // Max sign effect during HOLD_MAX status
                if current_status == 0x1E2 {
                    
                    if instance_key < 256 && (current_frame - LAST_MAX_SIGN_FRAME[instance_key] >= 15) {
                        let max_sign_config = EffectConfig::new("mewtwo_shadowball_max_sign", 15.0, 2.0, "body")
                            .with_params(1.0, 1.0, (0.0, 90.0, 0.0));
                        
                        let max_sign_key = format!("mewtwo_shadowball_max_sign_{}", current_frame);
                        manage_universal_effect(
                            fighter, boma, &max_sign_config, &max_sign_key,
                            current_frame, current_status, true, true
                        );
                        
                        LAST_MAX_SIGN_FRAME[instance_key] = current_frame;
                    }
                }
                
                // Suppress vanilla Purin effects when invisible
                let purin_effects = [
                    "purin_appeal_lw", "purin_attack_arc_d", "purin_final_bg_black",
                    "purin_final_bg_vortex", "purin_final_shockwave", "purin_final_smoke",
                    "purin_hataku", "purin_hataku_flash", "purin_hataku_hold",
                    "purin_korogaru", "purin_korogaru2", "purin_korogaru_loop",
                    "purin_korogaru_max", "purin_korogaru_wind", "purin_nemuru",
                    "purin_nemuru_end", "purin_nemuru_start", "purin_sleep",
                    "purin_smash_arc", "purin_smash_line", "purin_utau",
                    "sys_dash_smoke", "sys_flash"
                ];
                
                for effect_name in purin_effects.iter() {
                    EffectModule::kill_kind(boma, Hash40::new(effect_name), false, false);
                }
            }
        },
        
        // Charging below threshold - no special effects
        ShadowballState::ChargingBelowThreshold => {
            // Clean up any existing shadowball effects (per-player cleanup)
            if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
                let shadowball_keys: Vec<String> = effects.keys()
                    .filter(|k| k.contains("mewtwo_shadowball") && k.ends_with(&format!("_{}", instance_key)))
                    .cloned()
                    .collect();
                
                for key in shadowball_keys {
                    if let Some(tracker) = effects.remove(&key) {
                        EffectModule::kill(boma, tracker.handle, false, true);
                    }
                }
            }
        },
        
        // Not active - clean up everything
        ShadowballState::NotActive => {
            // Clean up effects when not in shadowball states (per-player cleanup)
            if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
                let shadowball_keys: Vec<String> = effects.keys()
                    .filter(|k| k.contains("mewtwo_shadowball") && k.ends_with(&format!("_{}", instance_key)))
                    .cloned()
                    .collect();
                
                for key in shadowball_keys {
                    if let Some(tracker) = effects.remove(&key) {
                        EffectModule::kill(boma, tracker.handle, false, true);
                    }
                }
            }
            
            // Reset shadowball tracking when completely out of shadowball
            player_state.shadowball_was_sufficiently_charged = false;
            player_state.shadowball_air_charge_count = 0;
        },
        
        // Transition state - keep model visible
        ShadowballState::TransitionKeepModel => {
            // Keep existing effects but don't add new ones
        }
    }
    
    // Speed booster effect ONLY during rollout if reached CHARGE MAX threshold
    let is_rollout = current_status == PURIN_SPECIAL_N_ROLL || 
                current_status == PURIN_SPECIAL_N_ROLL_AIR;

    if is_rollout {
        // Get enhanced charge detection values
        let official_charge_count = WorkModule::get_float(boma, PURIN_WORK_FLOAT_CHARGE_COUNT);
        let is_max_charge = WorkModule::is_flag(boma, PURIN_FLAG_MAX_FLAG);
        let hold_max_frames = WorkModule::get_int(boma, PURIN_WORK_INT_HOLD_MAX);
        
        // Only show speedbooster if reached CHARGE MAX threshold (not just invisibility threshold)
        let reached_charge_max_threshold = is_max_charge || 
                                        hold_max_frames > 0 ||
                                        (player_state.shadowball_previous_status == 0x1E2); // Was in HOLD_MAX status
        
        // NEW: Use shadowball state detection instead of direct hitbox check
        let shadowball_state = crate::gastly::visuals::detect_shadowball_hitbox_state(boma, player_state);
        let model_is_invisible = match shadowball_state {
            crate::gastly::visuals::ShadowballState::ChargedRolloutWithHitbox |
            crate::gastly::visuals::ShadowballState::RegularRolloutWithHitbox |
            crate::gastly::visuals::ShadowballState::AirToGroundRolloutWithHitbox => true,
            _ => false,
        };
        
        if reached_charge_max_threshold && model_is_invisible { // <- Use shadowball state instead
            
            if instance_key < 256 && (current_frame - LAST_SPEEDBOOSTER_FRAME[instance_key] >= 8) {
                // Spawn speed booster effect
                macros::EFFECT_FLW_POS(
                    fighter,
                    Hash40::new("sys_speedbooster"),
                    Hash40::new("top"),
                    0.0, 6.0, 3.0,
                    0.0, 0.0, 0.0,
                    0.8,
                    true
                );
                EffectModule::enable_sync_init_pos_last(fighter.module_accessor);
                macros::LAST_EFFECT_SET_COLOR(fighter, 0.7, 0.2, 1.0);
                macros::LAST_EFFECT_SET_ALPHA(fighter, 0.75);
                
                LAST_SPEEDBOOSTER_FRAME[instance_key] = current_frame;
            }
        }

            else if is_rollout {
            // Kill speedbooster when conditions no longer met during rollout
            EffectModule::kill_kind(boma, Hash40::new("sys_speedbooster"), false, true);
        }
    }
    
    // Bomb detection code
    let is_hit_end_status = current_status == 0x1E7;
    
    if is_hit_end_status {
        let status_just_changed = LAST_STATUS_FOR_BOMB[instance_key] != current_status;
        
        if status_just_changed {
            macros::EFFECT(
                fighter,
                Hash40::new("mewtwo_shadowball_bomb"),
                Hash40::new("body"),
                0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
                true
            );
            
            // MUTE VANILLA HIT SOUNDS FIRST
            SoundModule::stop_se(boma, Hash40::new("se_common_punch_hit_s"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_punch_hit_m"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_punch_hit_l"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_kick_hit_s"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_kick_hit_m"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_kick_hit_l"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_slap_hit_s"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_slap_hit_m"), 0);
            SoundModule::stop_se(boma, Hash40::new("se_common_slap_hit_l"), 0);
            
            // Play fire sound with volume 1.5
            let fire_handle = SoundModule::play_se(
                boma,
                Hash40::new("se_common_fire_m"),
                true, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, fire_handle as i32, 0.5, 0);
        }
    }
    
    LAST_STATUS_FOR_BOMB[instance_key] = current_status;
    
    // Reset bomb tracker when not in any shadowball-related status
    let is_any_shadowball_status = current_status == 0x1E1 || current_status == 0x1E2 || 
                                   current_status == 0x1E3 || current_status == 0x1E4 || 
                                   current_status == 0x1E5 || current_status == 0x1E6 || 
                                   current_status == 0x1E7;
    
    if !is_any_shadowball_status {
        // Reset tracking when completely out of shadowball
        player_state.shadowball_was_sufficiently_charged = false;
        player_state.shadowball_air_charge_count = 0;
    }
}

// CLEANUP FUNCTIONS
unsafe fn cleanup_gastly_aura(boma: *mut BattleObjectModuleAccessor) {
    let stored_handle = WorkModule::get_int(boma, GASTLY_AURA_HANDLE_WORK_ID) as u32;
    
    if stored_handle != 0 {
        EffectModule::kill(boma, stored_handle, false, true);
        WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID);
        
    }
}

pub unsafe fn kill_gastly_aura_on_evolution(boma: *mut BattleObjectModuleAccessor) {
    cleanup_gastly_aura(boma);
}

pub unsafe fn init_gastly_aura_handle(boma: *mut BattleObjectModuleAccessor) {
    // Clean up any existing effects
    cleanup_gastly_aura(boma);
    
    // Initialize work module values
    WorkModule::set_int(boma, 0, GASTLY_AURA_HANDLE_WORK_ID);
    WorkModule::set_float(boma, 0.0, FIGHTER_PURIN_INSTANCE_WORK_ID_FLOAT_GASTLY_AURA_FRAME);
    WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GASTLY_AURA_ACTIVE);
    
}

/// Clean up all universal effects for a specific player
pub unsafe fn cleanup_player_universal_effects(instance_key: u32) {
    if let Ok(mut effects) = UNIVERSAL_EFFECTS.lock() {
        let player_keys: Vec<String> = effects.keys()
            .filter(|k| k.ends_with(&format!("_{}", instance_key)))
            .cloned()
            .collect();
        
        // Get player's boma for effect killing
        let entry_id = (instance_key / 32) as u32; // Convert instance_key back to entry_id
        let boma = smash::app::sv_battle_object::module_accessor(entry_id);
        
        for key in player_keys {
            if let Some(tracker) = effects.remove(&key) {
                // Kill the visual effect if we have a valid boma and handle
                if !boma.is_null() && tracker.handle != u32::MAX && tracker.handle != 0 {
                    EffectModule::kill(boma, tracker.handle, false, true);
                }
            }
        }
    }
}

pub fn install_effects() {
}