// src/gastly/persist_sfx.rs - Evolution Sound Persistence System

use smash::app::lua_bind::{SoundModule, WorkModule, StatusModule};
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lib::lua_const::*;
use smash::lua2cpp::L2CFighterCommon;

// Import the constants from constants.rs instead of duplicating them
use crate::gastly::constants::*;


// Initialize all sound handles on fighter start
pub unsafe extern "C" fn init_evolution_sounds(fighter: &mut L2CFighterCommon) {
    let boma = fighter.module_accessor;
    
    // Initialize all sound handles to 0 (using the old handle system for non-looping sounds)
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SS_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_X_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_LINKING_CORD_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_DYNAMAX_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_GENGARITE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_CANCEL_EVOLVE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_GRAB_BURN_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_MEGASYMBOL_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_POTION_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_RESTORE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_SHADOWBALL_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_SHINY_SPARKLE_HANDLE);
}

// Condition sounds (these play once but shouldn't be interrupted)
pub unsafe fn play_condition_sound(boma: *mut BattleObjectModuleAccessor, condition_number: i32) {
    let (sound_name, volume) = match condition_number {
        1 => ("evolve_condition_1", 2.5),
        2 => ("evolve_condition_2", 2.5), 
        _ => return,
    };
    
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new(sound_name),
        false,
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, volume, 0);
}

// One-shot sound functions (these don't loop, but use the persist system for consistency)
pub unsafe fn play_evolve_se_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("evolve_se"),
        false, // Don't loop
        false,
        false, 
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.4, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SE_HANDLE);
}

pub unsafe fn play_evolve_ss_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("evolve_ss"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.0, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SS_HANDLE);
}

pub unsafe fn play_everstone_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("everstone"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.3, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_HANDLE);
}

pub unsafe fn play_everstone_x_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("everstone_x"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.8, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_X_HANDLE);
}

pub unsafe fn play_linking_cord_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("linking_cord"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.5, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_LINKING_CORD_HANDLE);
}

pub unsafe fn play_dynamax_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("dynamax"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.3, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_DYNAMAX_HANDLE);
}

pub unsafe fn play_gengarite_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("gengarite"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.7, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_GENGARITE_HANDLE);
}

pub unsafe fn play_cancel_evolve_sound(boma: *mut BattleObjectModuleAccessor) {
    let sound_handle = SoundModule::play_se(
        boma,
        Hash40::new("cancel_evolve"),
        false, // Don't loop
        false,
        false,
        false,
        smash::app::enSEType(0)
    );
    
    SoundModule::set_se_vol(boma, sound_handle as i32, 1.0, 0);
    WorkModule::set_int(boma, sound_handle as i32, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_CANCEL_EVOLVE_HANDLE);
}

// Clean up all sounds on death/respawn
pub unsafe fn cleanup_evolution_sounds_on_death(boma: *mut BattleObjectModuleAccessor) {

    // Reset all handles
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVOLVE_SS_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_EVERSTONE_X_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_LINKING_CORD_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_DYNAMAX_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_GENGARITE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_CANCEL_EVOLVE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_GRAB_BURN_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_MEGASYMBOL_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_POTION_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_RESTORE_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_G_SHADOWBALL_HANDLE);
    WorkModule::set_int(boma, 0, FIGHTER_PURIN_INSTANCE_WORK_ID_INT_SHINY_SPARKLE_HANDLE);
}

// Integration helper - call this from mod.rs to initialize the sound system
pub unsafe fn install_persistent_sound_system() {
    // Persistent sound system ready
}