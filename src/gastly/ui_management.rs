// src/gastly/ui_management.rs - New module for UI swapping and cutin effects

use smash::app::lua_bind::{WorkModule, StatusModule, SoundModule};
use smash::app::BattleObjectModuleAccessor;
use smash::app::utility;
use smash::lua2cpp::L2CFighterCommon;
use smash_script::macros;
use smash::phx::Hash40;
use smash::lib::lua_const::*;

use hash40::hash40;
use smash::hash40;

use crate::gastly::player_state::{PlayerEvolutionState, EvolutionStage};
use crate::gastly::FIGHTER_STATES;
use crate::gastly::constants::{FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY, ENABLE_EVOLUTION_CUTINS};

// Track original UI for restoration if needed
static mut ORIGINAL_UI_CHARA_HASH: [u64; 256] = [0x0; 256];

// Track last known evolution stage to detect changes
static mut LAST_EVOLUTION_STAGE: [u8; 256] = [0; 256]; // 0=Gastly, 1=Haunter, 2=Gengar

// Track cutin state to prevent spam
static mut CUTIN_PLAYED_THIS_EVOLUTION: [bool; 256] = [false; 256];
static mut CUTIN_PLAYED_MEGA: [bool; 256] = [false; 256];
static mut CUTIN_PLAYED_GIGA: [bool; 256] = [false; 256];

// Sound tracking for cutin triggers
static mut LAST_CRY_HAUNTER_FRAME: [i32; 256] = [-300; 256];
static mut LAST_CRY_GENGAR_FRAME: [i32; 256] = [-300; 256];

// Global UI death reset cooldown
static mut UI_DEATH_RESET_COOLDOWN: [i32; 256] = [0; 256];

// Track death frame to enforce stage reset for longer period
static mut DEATH_RESET_FRAME: [i32; 256] = [-1; 256];

// Changes the battle portrait (chara_4) UI based on current evolution stage
pub unsafe fn update_battle_portrait_ui(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState
) {
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }

    // Derive entry_id directly from THIS boma to ensure perfect match
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;

    // Check if we should delay UI changes for cutin display
    static mut CUTIN_RESTORE_TIMER: [i32; 256] = [0; 256];
    
    if CUTIN_RESTORE_TIMER[instance_key] > 0 {
        CUTIN_RESTORE_TIMER[instance_key] -= 1;
        return; // Skip UI changes while cutin is displaying
    }
    let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    
    // Check if character has changed (e.g., Fox -> Gastly in training mode)
    let current_fighter_kind = utility::get_kind(&mut *boma);
    let is_purin = current_fighter_kind == *FIGHTER_KIND_PURIN;
    
    // Reset stored UI if character switched away from Purin
    if !is_purin {
        if ORIGINAL_UI_CHARA_HASH[instance_key] != 0x0 {
            // Character switched away from Purin - restore the proper UI for current character
            let proper_ui_hash = match current_fighter_kind {
                k if k == *FIGHTER_KIND_FOX => hash40("ui_chara_fox"),
                k if k == *FIGHTER_KIND_FALCO => hash40("ui_chara_falco"),
                k if k == *FIGHTER_KIND_MARIO => hash40("ui_chara_mario"),
                k if k == *FIGHTER_KIND_DONKEY => hash40("ui_chara_donkey"),
                k if k == *FIGHTER_KIND_LINK => hash40("ui_chara_link"),
                k if k == *FIGHTER_KIND_SAMUS => hash40("ui_chara_samus"),
                k if k == *FIGHTER_KIND_YOSHI => hash40("ui_chara_yoshi"),
                k if k == *FIGHTER_KIND_KIRBY => hash40("ui_chara_kirby"),
                k if k == *FIGHTER_KIND_PIKACHU => hash40("ui_chara_pikachu"),
                k if k == *FIGHTER_KIND_LUIGI => hash40("ui_chara_luigi"),
                k if k == *FIGHTER_KIND_NESS => hash40("ui_chara_ness"),
                k if k == *FIGHTER_KIND_CAPTAIN => hash40("ui_chara_captain"),
                k if k == *FIGHTER_KIND_KOOPA => hash40("ui_chara_koopa"),
                k if k == *FIGHTER_KIND_PEACH => hash40("ui_chara_peach"),
                k if k == *FIGHTER_KIND_ZELDA => hash40("ui_chara_zelda"),
                k if k == *FIGHTER_KIND_SHEIK => hash40("ui_chara_sheik"),
                k if k == *FIGHTER_KIND_MARTH => hash40("ui_chara_marth"),
                k if k == *FIGHTER_KIND_GAMEWATCH => hash40("ui_chara_gamewatch"),
                _ => {
                    // For unknown characters, try to get their current UI hash
                    the_csk_collection_api::get_ui_chara_from_entry_id(entry_id)
                }
            };
            
            // Force restore proper UI for current character
            let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
            the_csk_collection_api::change_entry_chara_ui(
                entry_id,
                proper_ui_hash,
                owner_color as u8,
            );
            
            // Clear stored hash since we're no longer Purin
            ORIGINAL_UI_CHARA_HASH[instance_key] = 0x0;
        }
        return; // Exit early since we're not Purin anymore
    }
    
    // ROBUST ORIGINAL UI STORAGE - Only store once and protect from corruption
    if is_purin && ORIGINAL_UI_CHARA_HASH[instance_key] == 0x0 {
        // Always force ui_chara_purin as the original UI for all Purin instances
        // This ensures consistent behavior and prevents name text loss
        ORIGINAL_UI_CHARA_HASH[instance_key] = hash40("ui_chara_purin");
    }
    
    // PROTECTION: Prevent original UI from being overwritten during evolution
    // This is the key fix for name text disappearing - the original UI reference must never change
    if is_purin && ORIGINAL_UI_CHARA_HASH[instance_key] != 0x0 && ORIGINAL_UI_CHARA_HASH[instance_key] != hash40("ui_chara_purin") {
        // If original UI got corrupted somehow, restore it to ui_chara_purin
        ORIGINAL_UI_CHARA_HASH[instance_key] = hash40("ui_chara_purin");
    }
    
    // Determine which UI to use based on evolution stage and status
    let target_ui_hash = if player_state.is_evolving {
        // ★ Show "Who's that Pokemon?" UI during evolution
        hash40("ui_chara_evolving")
    } else {
        // Normal stage-based UI
        match player_state.stage {
            EvolutionStage::Gastly => {
                // Fallback to "ui_chara_purin" if original UI wasn't stored or is corrupted
                if ORIGINAL_UI_CHARA_HASH[instance_key] != 0x0 &&
                   ORIGINAL_UI_CHARA_HASH[instance_key] != hash40("ui_chara_evolving") &&
                   ORIGINAL_UI_CHARA_HASH[instance_key] != hash40("ui_chara_haunter") &&
                   ORIGINAL_UI_CHARA_HASH[instance_key] != hash40("ui_chara_gengar") {
                    ORIGINAL_UI_CHARA_HASH[instance_key]
                } else {
                    // PROTECTION: If original UI is corrupted or not stored, use Purin default
                    // and update the stored hash for future use
                    let fallback_ui = hash40("ui_chara_purin");
                    ORIGINAL_UI_CHARA_HASH[instance_key] = fallback_ui;
                    fallback_ui
                }
            },
            EvolutionStage::Haunter => hash40("ui_chara_haunter"),
            EvolutionStage::Gengar => hash40("ui_chara_gengar")
        }
    };
    
    // Get current UI hash to check if change is needed
    let current_ui_hash = the_csk_collection_api::get_ui_chara_from_entry_id(entry_id);

    // Debug logging for marked costumes
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    let is_marked_costume = if color_id < 256 {
        crate::MARKED_COLORS[color_id]
    } else {
        false
    };

    // BULLETPROOF STAGE CHANGE DETECTION: Force UI refresh when evolution stage changes
    let current_stage_value = match player_state.stage {
        EvolutionStage::Gastly => 0,
        EvolutionStage::Haunter => 1,
        EvolutionStage::Gengar => 2,
    };
    
    let last_stage = LAST_EVOLUTION_STAGE[instance_key];
    let stage_changed = last_stage != current_stage_value;
    
    if stage_changed {
        // Evolution stage changed - force UI refresh by clearing current UI knowledge
        LAST_EVOLUTION_STAGE[instance_key] = current_stage_value;
        
        // Force immediate UI update by ensuring target differs from current
        let current_ui_hash = the_csk_collection_api::get_ui_chara_from_entry_id(entry_id);
        let target_ui_hash = match player_state.stage {
            EvolutionStage::Gastly => {
                if ORIGINAL_UI_CHARA_HASH[instance_key] != 0x0 {
                    ORIGINAL_UI_CHARA_HASH[instance_key]
                } else {
                    hash40("ui_chara_purin")
                }
            },
            EvolutionStage::Haunter => hash40("ui_chara_haunter"),
            EvolutionStage::Gengar => hash40("ui_chara_gengar"),
        };
        
        // Force the UI change immediately on stage change
        if current_ui_hash != target_ui_hash {
            the_csk_collection_api::change_entry_chara_ui(
                entry_id,
                target_ui_hash,
                owner_color as u8,
            );
        }
        return; // Skip normal UI update logic after forced refresh
    }
    
    // Check 3: For marked costumes, add extra validation
    if is_marked_costume {
        // Verify this is actually a Purin character
        let current_kind = utility::get_kind(&mut *boma);
        if current_kind != *FIGHTER_KIND_PURIN {
            return; // Abort - not even Purin
        }
        
        // Verify the color matches what we expect
        let boma_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
        if boma_color != color_id {
            return; // Abort - color mismatch
        }
    }
    
    // Only change if different
    if current_ui_hash != target_ui_hash {
        the_csk_collection_api::change_entry_chara_ui(
            entry_id,
            target_ui_hash,
            owner_color as u8,
        );
    }
}

/// Handles cutin effects with appropriate chara_6 UI for different forms
pub unsafe fn handle_cutin_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
) {
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    // BULLETPROOF ISOLATION: Derive entry_id from THIS boma
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    if !ENABLE_EVOLUTION_CUTINS {
        // Clear any pending cutin flags when disabled
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
        return;
    }
    
    let current_frame = player_state.current_frame;
    
    // Check for cry sound triggers for evolution cutins
    check_for_evolution_cry_cutins(boma, player_state, fighter, instance_key, current_frame);
    
    // Check for final smash form cutins
    check_for_final_smash_cutins(boma, player_state, fighter, instance_key);
    
    // Check for immediate evolution cutin flags
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
        let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        trigger_evolution_cutin(fighter, "Haunter", boma_entry_id);
    }
    
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
        let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        trigger_evolution_cutin(fighter, "Gengar", boma_entry_id);
    }
}

/// Checks for cry_haunter and cry_gengar sounds to trigger evolution cutins
unsafe fn check_for_evolution_cry_cutins(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    instance_key: usize,
    current_frame: i32
) {
    // Check if we just completed evolution to Haunter
    if player_state.evolution_just_completed_this_frame && 
       player_state.stage == EvolutionStage::Haunter &&
       !CUTIN_PLAYED_THIS_EVOLUTION[instance_key] {
        
        // Check if cry_haunter sound was played recently (within last 30 frames)
        let frames_since_last_cry = current_frame - LAST_CRY_HAUNTER_FRAME[instance_key];
        if frames_since_last_cry <= 30 && frames_since_last_cry >= 0 {
            // Use the derived entry_id from this boma
            let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
            trigger_evolution_cutin(fighter, "Haunter", boma_entry_id);
            CUTIN_PLAYED_THIS_EVOLUTION[instance_key] = true;
        }
    }
    
    // Check if we just completed evolution to Gengar
    if player_state.evolution_just_completed_this_frame && 
       player_state.stage == EvolutionStage::Gengar &&
       !CUTIN_PLAYED_THIS_EVOLUTION[instance_key] {
        
        // Check if cry_gengar sound was played recently (within last 30 frames)
        let frames_since_last_cry = current_frame - LAST_CRY_GENGAR_FRAME[instance_key];
        if frames_since_last_cry <= 30 && frames_since_last_cry >= 0 {
            // Use the derived entry_id from this boma
            let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
            trigger_evolution_cutin(fighter, "Gengar", boma_entry_id);
            CUTIN_PLAYED_THIS_EVOLUTION[instance_key] = true;
        }
    }
    
    // Reset evolution cutin flag when starting new evolution
    if player_state.is_evolving && CUTIN_PLAYED_THIS_EVOLUTION[instance_key] {
        CUTIN_PLAYED_THIS_EVOLUTION[instance_key] = false;
    }
}

/// Checks for final smash form activation to trigger mega/giga cutins
unsafe fn check_for_final_smash_cutins(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    instance_key: usize
) {
    let is_final_smash = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
    let current_status = StatusModule::status_kind(boma);
    
    // Trigger Mega Gengar cutin when mega form becomes active during final smash
    if is_final_smash && 
       player_state.mega_gengar_form_active && 
       player_state.is_in_final_smash_form &&
       current_status == 0x1E0 && // FINAL status
       !CUTIN_PLAYED_MEGA[instance_key] {
        
        let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        trigger_final_smash_cutin(fighter, "Mega Gengar", boma_entry_id);
        CUTIN_PLAYED_MEGA[instance_key] = true;
    }
    
    // Trigger Gigantamax Gengar cutin when giga form becomes active during final smash
    if is_final_smash && 
       player_state.giga_gengar_form_active && 
       player_state.is_in_final_smash_form &&
       current_status == 0x1E0 && // FINAL status
       !CUTIN_PLAYED_GIGA[instance_key] {
        
        let boma_entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize;
        trigger_final_smash_cutin(fighter, "Gigantamax Gengar", boma_entry_id);
        CUTIN_PLAYED_GIGA[instance_key] = true;
    }
    
    // Reset final smash cutin flags when final smash ends
    if !is_final_smash && (CUTIN_PLAYED_MEGA[instance_key] || CUTIN_PLAYED_GIGA[instance_key]) {
        CUTIN_PLAYED_MEGA[instance_key] = false;
        CUTIN_PLAYED_GIGA[instance_key] = false;
    }
}

/// Triggers cutin effect for evolution with appropriate chara_6 UI
unsafe fn trigger_evolution_cutin(
    fighter: &mut L2CFighterCommon,
    evolution_name: &str,
    entry_id: usize
) {
    // Temporarily change to appropriate chara_6 UI for cutin
    let cutin_ui_hash = match evolution_name {
        "Haunter" => hash40("ui_chara_haunter_00"),
        "Gengar" => hash40("ui_chara_gengar_00"),
        _ => return,
    };
    
    // Change to cutin UI temporarily
    let boma = fighter.module_accessor;
    let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    let entry_id_u32 = entry_id as u32;
    
    the_csk_collection_api::change_entry_chara_ui(
        entry_id_u32,
        cutin_ui_hash,
        owner_color as _,
    );
    
    // Trigger cutin effect
    macros::FT_START_CUTIN(fighter);
    
    
    // Schedule restoration of chara_4 UI after cutin (will happen in next frame via update_battle_portrait_ui)
}

/// Triggers cutin effect for final smash forms with mega/giga chara_6 UI
unsafe fn trigger_final_smash_cutin(
    fighter: &mut L2CFighterCommon,
    form_name: &str,
    entry_id: usize
) {
    // Temporarily change to appropriate chara_6 UI for cutin
    let cutin_ui_hash = match form_name {
        "Mega Gengar" => hash40("ui_chara_mega_gengar_00"),
        "Gigantamax Gengar" => hash40("ui_chara_giga_gengar_00"),
        _ => return,
    };
    
    // Change to cutin UI temporarily
    let boma = fighter.module_accessor;
    let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    let entry_id_u32 = entry_id as u32;
    
    the_csk_collection_api::change_entry_chara_ui(
        entry_id_u32,
        cutin_ui_hash,
        owner_color as _,
    );
    
    // Trigger cutin effect
    macros::FT_START_CUTIN(fighter);

    // Delay restoration to let cutin display - use entry_id for this timer since it's for UI API
    static mut CUTIN_RESTORE_TIMER_FINAL_SMASH: [i32; 8] = [0; 8];
    if entry_id < 8 {
        CUTIN_RESTORE_TIMER_FINAL_SMASH[entry_id] = 120; // 2 second delay
    }
    
}

/// Tracks cry sound playback for cutin timing
pub unsafe fn track_cry_sound_playback(
    sound_hash: Hash40,
    current_frame: i32,
    entry_id: u32
) {
    // Convert entry_id to instance_key for proper tracking
    // Since we only have entry_id here, we need to find the corresponding boma
    let boma = smash::app::sv_battle_object::module_accessor(entry_id);
    if boma.is_null() { return; }
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    if sound_hash.hash == smash::hash40("cry_haunter") {
        LAST_CRY_HAUNTER_FRAME[instance_key] = current_frame;
    } else if sound_hash.hash == smash::hash40("cry_gengar") {
        LAST_CRY_GENGAR_FRAME[instance_key] = current_frame;
    }
}

/// Resets UI state on death/respawn
pub unsafe fn reset_ui_state_on_death(entry_id: u32) {
    // Convert entry_id to instance_key for proper tracking
    let boma = smash::app::sv_battle_object::module_accessor(entry_id);
    if boma.is_null() { return; }
    
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    if instance_key >= 256 { return; }
    
    // Reset cutin flags
    CUTIN_PLAYED_THIS_EVOLUTION[instance_key] = false;
    CUTIN_PLAYED_MEGA[instance_key] = false;
    CUTIN_PLAYED_GIGA[instance_key] = false;
    
    // Reset cry tracking
    LAST_CRY_HAUNTER_FRAME[instance_key] = -300;
    LAST_CRY_GENGAR_FRAME[instance_key] = -300;
    
    // Reset stage tracking to force UI refresh on next update
    LAST_EVOLUTION_STAGE[instance_key] = 255; // Invalid value to trigger stage change detection
    
    // Clear UI state on death
    ORIGINAL_UI_CHARA_HASH[instance_key] = 0x0;
}

/// Main UI management function called from main loop
pub unsafe fn handle_ui_management(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon
) {
    // Get instance key and color info
    let instance_key = crate::gastly::get_instance_key(boma) as usize;
    let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    let color_id = owner_color as usize;
    let is_marked_costume = if color_id < 256 {
        crate::MARKED_COLORS[color_id]
    } else {
        false
    };
    
    // DISABLED: Aggressive death reset enforcement was causing UI interference with other players
    // The core issue is that the death reset logic needs to be less intrusive
    
    // DISABLED: Death reset cooldown was causing UI interference
    // Let normal UI management handle all cases without artificial delays

    // Reduce UI update frequency to prevent flickering (only update every 10 frames unless evolving)
    static mut LAST_UI_UPDATE_FRAME: [i32; 256] = [-10; 256];
    if instance_key < 256 {
        let frames_since_last_update = player_state.current_frame - LAST_UI_UPDATE_FRAME[instance_key];
        let should_update_ui = player_state.is_evolving || 
                              player_state.evolution_just_completed_this_frame ||
                              frames_since_last_update >= 10;
        
        if !should_update_ui {
            // Still handle cutins even if we skip UI updates
            handle_cutin_effects(boma, player_state, fighter);
            return;
        }
        
        LAST_UI_UPDATE_FRAME[instance_key] = player_state.current_frame;
    }
    
    // Update battle portrait UI based on current evolution stage
    update_battle_portrait_ui(boma, player_state);
    
    // Handle cutin effects for evolutions and final smash forms
    handle_cutin_effects(boma, player_state, fighter);

    /// Direct cutin trigger for evolution (bypasses timing checks)
    pub unsafe fn trigger_evolution_cutin_direct(
        fighter: &mut L2CFighterCommon,
        evolution_name: &str,
        entry_id: usize
    ) {
        let cutin_ui_hash = match evolution_name {
            "Haunter" => hash40("ui_chara_haunter_00"),
            "Gengar" => hash40("ui_chara_gengar_00"),
            _ => return,
        };
        
        let boma = fighter.module_accessor;
        let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
        let entry_id_u32 = entry_id as u32;
        
        // Change to cutin UI
        the_csk_collection_api::change_entry_chara_ui(
            entry_id_u32,
            cutin_ui_hash,
            owner_color as _,
        );
        
        
        // Trigger cutin effect  
        macros::FT_START_CUTIN(fighter);
        
    }
}