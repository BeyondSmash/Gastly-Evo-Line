// src/gastly/sounds.rs

use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lua2cpp::L2CFighterCommon;
use smash_script::macros;
use smash_script::sound;
use smash_script::lua_args;
use smashline::*;
use smash::app::sv_animcmd::*;

// Import our player state system
use crate::gastly::{FIGHTER_STATES, player_state::EvolutionStage};


// Helper function to determine if current stage should mute step sounds
unsafe fn should_mute_steps(boma: *mut BattleObjectModuleAccessor) -> bool {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let states_map = FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&entry_id) {
        matches!(player_state.stage, 
                EvolutionStage::Gastly | 
                EvolutionStage::Haunter)
    } else {
        true // Default to Gastly (mute steps)
    }
}

// Walk Slow - Back to working PLAY_STATUS for levitation, proper vanilla for Gengar
unsafe extern "C" fn sound_walkslow(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let lua_state = agent.lua_state_agent;
    
    if should_mute_steps(boma) {
        // Gastly/Haunter: Use working PLAY_STATUS approach
        frame(lua_state, 1.0);
        if macros::is_excute(agent) {
            sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
            macros::PLAY_STATUS(agent, Hash40::new("g_walkslow"));
        }
    } else {
        // Gengar: Use looping step sounds
        loop {
            wait_loop_sync_mot(lua_state);
            frame(lua_state, 23.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_s"));
            }
            wait(lua_state, 20.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_left_s"));
            }
        }
    }
}

// Walk Middle - Back to working PLAY_STATUS for levitation, proper vanilla for Gengar
unsafe extern "C" fn sound_walkmiddle(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let lua_state = agent.lua_state_agent;
    
    if should_mute_steps(boma) {
        // Gastly/Haunter: Use working PLAY_STATUS approach
        frame(lua_state, 1.0);
        if macros::is_excute(agent) {
            sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
            macros::PLAY_STATUS(agent, Hash40::new("g_walkmiddle"));
        }
    } else {
        // Gengar: Use looping step sounds
        loop {
            frame(lua_state, 7.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_m"));
            }
            wait(lua_state, 23.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_left_s"));
            }
            wait_loop_sync_mot(lua_state);
            frame(lua_state, 23.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_s"));
            }
            wait(lua_state, 20.0);
        }
    }
}

// Walk Fast - Back to working PLAY_STATUS for levitation, proper vanilla for Gengar
unsafe extern "C" fn sound_walkfast(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let lua_state = agent.lua_state_agent;
    
    if should_mute_steps(boma) {
        // Gastly/Haunter: Use working PLAY_STATUS approach
        frame(lua_state, 1.0);
        if macros::is_excute(agent) {
            sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
            macros::PLAY_STATUS(agent, Hash40::new("g_walkfast"));
        }
    } else {
        // Gengar: Use looping step sounds
        loop {
            frame(lua_state, 6.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_m"));
            }
            wait(lua_state, 15.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_left_s"));
            }
            wait_loop_sync_mot(lua_state);
            frame(lua_state, 23.0);
            if macros::is_excute(agent) {
                macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_s"));
            }
            wait(lua_state, 20.0);
        }
    }
}

// Run - Back to working PLAY_STATUS for all stages
unsafe extern "C" fn sound_run(agent: &mut L2CAgentBase) {
    let boma = agent.module_accessor;
    let lua_state = agent.lua_state_agent;
    
    // All stages use looping run sound (back to working PLAY_STATUS approach)
    frame(lua_state, 1.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("g_run"));
    }
}

// Simple cleanup function - PLAY_STATUS handles cleanup automatically
pub unsafe fn cleanup_motion_sounds_on_death(boma: *mut BattleObjectModuleAccessor) {
    // Status sounds are automatically cleaned up by the game
    SoundModule::stop_se(boma, Hash40::new("g_run"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_walkfast"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_walkmiddle"), 0);
    SoundModule::stop_se(boma, Hash40::new("g_walkslow"), 0);
}

// Install custom sound logic
pub fn install_sound_logic_with_costumes(costume: &[usize]) {
    Agent::new("purin")
        .set_costume(costume.to_vec())
        .sound_acmd("sound_walkslow", sound_walkslow, Priority::Low)
        .sound_acmd("sound_walkmiddle", sound_walkmiddle, Priority::Low)
        .sound_acmd("sound_walkfast", sound_walkfast, Priority::Low)
        .sound_acmd("sound_run", sound_run, Priority::Low)
        .install();
    
}