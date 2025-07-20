// src/gastly/ui_management.rs - New module for UI swapping and cutin effects

use smash::app::lua_bind::{WorkModule, StatusModule, SoundModule};
use smash::app::BattleObjectModuleAccessor;
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
static mut ORIGINAL_UI_CHARA_HASH: [u64; 8] = [0x0; 8];

// Track cutin state to prevent spam
static mut CUTIN_PLAYED_THIS_EVOLUTION: [bool; 8] = [false; 8];
static mut CUTIN_PLAYED_MEGA: [bool; 8] = [false; 8];
static mut CUTIN_PLAYED_GIGA: [bool; 8] = [false; 8];

// Sound tracking for cutin triggers
static mut LAST_CRY_HAUNTER_FRAME: [i32; 8] = [-300; 8];
static mut LAST_CRY_GENGAR_FRAME: [i32; 8] = [-300; 8];

// Changes the battle portrait (chara_4) UI based on current evolution stage
pub unsafe fn update_battle_portrait_ui(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    entry_id: u32
) {
    if entry_id >= 8 { return; }

    // Check if we should delay UI changes for cutin display
    static mut CUTIN_RESTORE_TIMER: [i32; 8] = [0; 8];
    let entry_id_usize = entry_id as usize;
    
    if CUTIN_RESTORE_TIMER[entry_id_usize] > 0 {
        CUTIN_RESTORE_TIMER[entry_id_usize] -= 1;
        return; // Skip UI changes while cutin is displaying
    }
    
    let entry_id_usize = entry_id as usize;
    let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
    
    // Store original UI hash if not already stored
    if ORIGINAL_UI_CHARA_HASH[entry_id_usize] == 0x0 {
        ORIGINAL_UI_CHARA_HASH[entry_id_usize] = the_csk_collection_api::get_ui_chara_from_entry_id(entry_id);
        println!("[UI MANAGEMENT] Stored original UI hash for entry {}: {:#x}", entry_id, ORIGINAL_UI_CHARA_HASH[entry_id_usize]);
    }
    
    // Determine which UI to use based on evolution stage and status
    let target_ui_hash = if player_state.is_evolving {
        // ★ Show "Who's that Pokemon?" UI during evolution
        hash40("ui_chara_evolving")
    } else {
        // Normal stage-based UI
        match player_state.stage {
            EvolutionStage::Gastly => ORIGINAL_UI_CHARA_HASH[entry_id_usize],
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

    // Only change if different (this should rarely trigger since we're keeping original)
    if current_ui_hash != target_ui_hash {
        the_csk_collection_api::change_entry_chara_ui(
            entry_id,
            target_ui_hash,
            owner_color as u8,
        );
        
        println!("[UI MANAGEMENT] ★ Restored original Purin UI to preserve stock icon (hash: {:#x})", target_ui_hash);
        
        // CRITICAL: The portraits will change via the layout DB entries we registered in lib.rs
        // but the stock icon will remain as Purin since we're using ORIGINAL_UI_CHARA_HASH
        
        let stage_name = if player_state.is_evolving {
            "Evolving (Who's that Pokemon?)"
        } else {
            match player_state.stage {
                EvolutionStage::Gastly => "Gastly (original)",
                EvolutionStage::Haunter => "Haunter",
                EvolutionStage::Gengar => "Gengar",
            }
        };
    }
}

/// Handles cutin effects with appropriate chara_6 UI for different forms
pub unsafe fn handle_cutin_effects(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    entry_id: u32
) {
    if !ENABLE_EVOLUTION_CUTINS {
        // Clear any pending cutin flags when disabled
        if entry_id < 8 {
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
            WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
        }
        return;
    }
    
    if entry_id >= 8 { return; }
    
    let entry_id_usize = entry_id as usize;
    let current_frame = player_state.current_frame;
    
    // Check for cry sound triggers for evolution cutins
    check_for_evolution_cry_cutins(boma, player_state, fighter, entry_id_usize, current_frame);
    
    // Check for final smash form cutins
    check_for_final_smash_cutins(boma, player_state, fighter, entry_id_usize);
    
    // Check for immediate evolution cutin flags
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
        trigger_evolution_cutin(fighter, "Haunter", entry_id as usize);
        println!("[UI MANAGEMENT] ★ IMMEDIATE CUTIN ★ Triggered Haunter cutin from flag");
    }
    
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
        trigger_evolution_cutin(fighter, "Gengar", entry_id as usize);
        println!("[UI MANAGEMENT] ★ IMMEDIATE CUTIN ★ Triggered Gengar cutin from flag");
    }
}

/// Checks for cry_haunter and cry_gengar sounds to trigger evolution cutins
unsafe fn check_for_evolution_cry_cutins(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    entry_id: usize,
    current_frame: i32
) {
    // Check if we just completed evolution to Haunter
    if player_state.evolution_just_completed_this_frame && 
       player_state.stage == EvolutionStage::Haunter &&
       !CUTIN_PLAYED_THIS_EVOLUTION[entry_id] {
        
        // Check if cry_haunter sound was played recently (within last 30 frames)
        let frames_since_last_cry = current_frame - LAST_CRY_HAUNTER_FRAME[entry_id];
        if frames_since_last_cry <= 30 && frames_since_last_cry >= 0 {
            trigger_evolution_cutin(fighter, "Haunter", entry_id);
            CUTIN_PLAYED_THIS_EVOLUTION[entry_id] = true;
        }
    }
    
    // Check if we just completed evolution to Gengar
    if player_state.evolution_just_completed_this_frame && 
       player_state.stage == EvolutionStage::Gengar &&
       !CUTIN_PLAYED_THIS_EVOLUTION[entry_id] {
        
        // Check if cry_gengar sound was played recently (within last 30 frames)
        let frames_since_last_cry = current_frame - LAST_CRY_GENGAR_FRAME[entry_id];
        if frames_since_last_cry <= 30 && frames_since_last_cry >= 0 {
            trigger_evolution_cutin(fighter, "Gengar", entry_id);
            CUTIN_PLAYED_THIS_EVOLUTION[entry_id] = true;
        }
    }
    
    // Reset evolution cutin flag when starting new evolution
    if player_state.is_evolving && CUTIN_PLAYED_THIS_EVOLUTION[entry_id] {
        CUTIN_PLAYED_THIS_EVOLUTION[entry_id] = false;
        println!("[UI MANAGEMENT] Reset evolution cutin flag for entry {} - new evolution started", entry_id);
    }

    // Check for immediate evolution cutin flags
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_HAUNTER_CUTIN_READY);
        trigger_evolution_cutin(fighter, "Haunter", entry_id as usize);
        println!("[UI MANAGEMENT] ★ IMMEDIATE CUTIN ★ Triggered Haunter cutin from flag");
    }
    
    if WorkModule::is_flag(boma, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY) {
        WorkModule::set_flag(boma, false, FIGHTER_PURIN_INSTANCE_WORK_ID_FLAG_GENGAR_CUTIN_READY);
        trigger_evolution_cutin(fighter, "Gengar", entry_id as usize);
        println!("[UI MANAGEMENT] ★ IMMEDIATE CUTIN ★ Triggered Gengar cutin from flag");
    }
}

/// Checks for final smash form activation to trigger mega/giga cutins
unsafe fn check_for_final_smash_cutins(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    entry_id: usize
) {
    let is_final_smash = WorkModule::is_flag(boma, *FIGHTER_INSTANCE_WORK_ID_FLAG_FINAL);
    let current_status = StatusModule::status_kind(boma);
    
    // Trigger Mega Gengar cutin when mega form becomes active during final smash
    if is_final_smash && 
       player_state.mega_gengar_form_active && 
       player_state.is_in_final_smash_form &&
       current_status == 0x1E0 && // FINAL status
       !CUTIN_PLAYED_MEGA[entry_id] {
        
        trigger_final_smash_cutin(fighter, "Mega Gengar", entry_id);
        CUTIN_PLAYED_MEGA[entry_id] = true;
    }
    
    // Trigger Gigantamax Gengar cutin when giga form becomes active during final smash
    if is_final_smash && 
       player_state.giga_gengar_form_active && 
       player_state.is_in_final_smash_form &&
       current_status == 0x1E0 && // FINAL status
       !CUTIN_PLAYED_GIGA[entry_id] {
        
        trigger_final_smash_cutin(fighter, "Gigantamax Gengar", entry_id);
        CUTIN_PLAYED_GIGA[entry_id] = true;
    }
    
    // Reset final smash cutin flags when final smash ends
    if !is_final_smash && (CUTIN_PLAYED_MEGA[entry_id] || CUTIN_PLAYED_GIGA[entry_id]) {
        CUTIN_PLAYED_MEGA[entry_id] = false;
        CUTIN_PLAYED_GIGA[entry_id] = false;
        println!("[UI MANAGEMENT] Reset final smash cutin flags for entry {} - final smash ended", entry_id);
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
    
    println!("[UI MANAGEMENT] ★ CUTIN DEBUG ★ UI changed to: {:#x}, Color: {}, Entry: {}", 
            cutin_ui_hash, owner_color, entry_id);
    
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

    println!("[UI MANAGEMENT] ★ FINAL SMASH CUTIN ★ Triggered {} cutin for entry {} with chara_6 UI", 
                form_name, entry_id);
    // Delay restoration to let cutin display
    static mut CUTIN_RESTORE_TIMER: [i32; 8] = [0; 8];
    CUTIN_RESTORE_TIMER[entry_id] = 120; // 2 second delay
    
    println!("[UI MANAGEMENT] ★ FINAL SMASH CUTIN ★ Set restore timer for entry {}", entry_id);
}

/// Tracks cry sound playback for cutin timing
pub unsafe fn track_cry_sound_playback(
    sound_hash: Hash40,
    current_frame: i32,
    entry_id: u32
) {
    if entry_id >= 8 { return; }
    
    let entry_id_usize = entry_id as usize;
    
    if sound_hash.hash == smash::hash40("cry_haunter") {
        LAST_CRY_HAUNTER_FRAME[entry_id_usize] = current_frame;
        println!("[UI MANAGEMENT] TRACKED cry_haunter sound at frame {} for entry {}", current_frame, entry_id);
    } else if sound_hash.hash == smash::hash40("cry_gengar") {
        LAST_CRY_GENGAR_FRAME[entry_id_usize] = current_frame;
        println!("[UI MANAGEMENT] TRACKED cry_gengar sound at frame {} for entry {}", current_frame, entry_id);
    }
}

/// Resets UI state on death/respawn
pub unsafe fn reset_ui_state_on_death(entry_id: u32) {
    if entry_id >= 8 { return; }
    
    let entry_id_usize = entry_id as usize;
    
    // Reset cutin flags
    CUTIN_PLAYED_THIS_EVOLUTION[entry_id_usize] = false;
    CUTIN_PLAYED_MEGA[entry_id_usize] = false;
    CUTIN_PLAYED_GIGA[entry_id_usize] = false;
    
    // Reset cry tracking
    LAST_CRY_HAUNTER_FRAME[entry_id_usize] = -300;
    LAST_CRY_GENGAR_FRAME[entry_id_usize] = -300;
    
    // Restore original UI if we have it stored
    if ORIGINAL_UI_CHARA_HASH[entry_id_usize] != 0x0 {
        let current_ui = the_csk_collection_api::get_ui_chara_from_entry_id(entry_id);
        if current_ui != ORIGINAL_UI_CHARA_HASH[entry_id_usize] {
            // Get color for restoration
            if let Ok(boma) = (|| -> Result<*mut BattleObjectModuleAccessor, ()> {
                let boma = smash::app::sv_battle_object::module_accessor(entry_id);
                if boma.is_null() { return Err(()); }
                Ok(boma)
            })() {
                let owner_color = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR);
                
                the_csk_collection_api::change_entry_chara_ui(
                    entry_id,
                    ORIGINAL_UI_CHARA_HASH[entry_id_usize],
                    owner_color as u8,
                );
                
                println!("[UI MANAGEMENT] Restored original UI for entry {} on death (hash: {:#x})", 
                        entry_id, ORIGINAL_UI_CHARA_HASH[entry_id_usize]);
            }
        }
    }
    
    println!("[UI MANAGEMENT] Reset UI state for entry {} on death/respawn", entry_id);
}

/// Main UI management function called from main loop
pub unsafe fn handle_ui_management(
    boma: *mut BattleObjectModuleAccessor,
    player_state: &PlayerEvolutionState,
    fighter: &mut L2CFighterCommon,
    entry_id: u32
) {
    // CRITICAL: Block UI changes for marked costumes during early frames
    let color_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    let is_marked_costume = if color_id < 256 {
        crate::MARKED_COLORS[color_id]
    } else {
        false
    };
    
    if is_marked_costume && player_state.current_frame < 120 {
        // Force Gastly UI during early frames for marked costumes
        if player_state.stage != EvolutionStage::Gastly {
            println!("[UI BLOCK] Blocking UI change for marked costume c{:02} during early frames", color_id);
            return;
        }
    }
    // Update battle portrait UI based on current evolution stage
    update_battle_portrait_ui(boma, player_state, entry_id);
    
    // Handle cutin effects for evolutions and final smash forms
    handle_cutin_effects(boma, player_state, fighter, entry_id);

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
        
        println!("[UI MANAGEMENT] ★ DIRECT CUTIN ★ Changed to chara_6: {:#x} for {}", 
                cutin_ui_hash, evolution_name);
        
        // Trigger cutin effect  
        macros::FT_START_CUTIN(fighter);
        
        println!("[UI MANAGEMENT] ★ DIRECT CUTIN ★ Triggered {} evolution cutin", evolution_name);
    }
}