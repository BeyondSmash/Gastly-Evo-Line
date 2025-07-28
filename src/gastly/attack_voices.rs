// Attack Voices - Part 1: Play attack voice AFTER PLAY_SEQUENCE

use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lua2cpp::L2CFighterCommon;
use smash_script::macros;
use smashline::*;
use smash::app::sv_animcmd::*;

// Import our sound system
use crate::gastly::{FIGHTER_STATES};
use crate::gastly::player_state::EvolutionStage;

// Helper function to get stage-specific attack voice
unsafe fn get_attack_voice_correct(boma: *mut BattleObjectModuleAccessor, attack_type: &str) -> &'static str {
    let instance_key = crate::gastly::get_instance_key(boma);
    
    let states_map = FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&instance_key) {
        match player_state.stage {
            crate::gastly::player_state::EvolutionStage::Gastly => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "gas_attack01",
                    2 => "gas_attack02_07", // Correct name!
                    3 => "gas_attack03",
                    4 => "gas_attack06",
                    _ => "gas_attack01",
                }
            },
            crate::gastly::player_state::EvolutionStage::Haunter => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "hau_attack01",
                    2 => "hau_attack02_07", // Correct name!
                    3 => "hau_attack03",
                    4 => "hau_attack06",
                    _ => "hau_attack01",
                }
            },
            crate::gastly::player_state::EvolutionStage::Gengar => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "gen_attack01",
                    2 => "gen_attack02_07", // Correct name!
                    3 => "gen_attack03",
                    4 => "gen_attack06",
                    _ => "gen_attack01",
                }
            },
        }
    } else {
        "gas_attack01" // Default
    }
}

// Back Air - Updated with 20% chance
unsafe extern "C" fn sound_attackairb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 10.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Up Air - Updated with 20% chance
unsafe extern "C" fn sound_attackairhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 8.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack01"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 9.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Dash Attack - Updated with 20% chance
unsafe extern "C" fn sound_attackdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 8.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 9.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Up Tilt - Updated with 20% chance
unsafe extern "C" fn sound_attackhi3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 10.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Attack Voices - Part 2: Remaining attacks with AFTER timing

// Forward Tilt - Updated with 20% chance
unsafe extern "C" fn sound_attacks3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 10.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}


// Forward Tilt Hi - Play attack voice AFTER PLAY_SEQUENCE
unsafe extern "C" fn sound_attacks3hi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 7.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 8.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack02");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Forward Tilt Lw - Play attack voice AFTER PLAY_SEQUENCE
unsafe extern "C" fn sound_attacks3lw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 7.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack02"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 8.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack02");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Side Special Air - Play attack voice AFTER vanilla voice
unsafe extern "C" fn sound_specialairs(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_s01"));
    }
    
    frame(lua_state, 12.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Side Special Ground - Play attack voice AFTER vanilla voice
unsafe extern "C" fn sound_specials(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_s01"));
    }
    
    frame(lua_state, 12.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Forward Smash - Updated with 20% chance
unsafe extern "C" fn sound_attacks4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 17.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_smash_start_04"));
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack_smash_l"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_ll"));
    }
    
    frame(lua_state, 18.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 1.2, 0);
        }
    }
}

// Up Smash - Converted with 20% attack voice chance
unsafe extern "C" fn sound_attackhi4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 16.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_smash_start_04"));
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack_smash_h"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_ll"));
    }
    
    //  Add attack voice 1 frame after PLAY_SEQUENCE
    frame(lua_state, 17.0);
    if macros::is_excute(agent) {
        let voice_chance = crate::gastly::random_module::rand_range_i32(1, 100);
        if voice_chance <= 35 {
            let attack_sound = get_attack_voice_correct(boma, "attack");
            let attack_handle = SoundModule::play_se(
                boma,
                Hash40::new(attack_sound),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, attack_handle as i32, 2.5, 0);
        }
    }
    
    wait(lua_state, 4.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_step_left_m"));
    }
    wait(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_step_right_m"));
    }
}

// Installation function
pub fn install_attack_voices_remaining_with_costumes(costume: &[usize]) {
    Agent::new("purin") 
        .set_costume(costume.to_vec())
        .sound_acmd("sound_attackairb", sound_attackairb, Priority::Low)
        .sound_acmd("sound_attackairhi", sound_attackairhi, Priority::Low)
        .sound_acmd("sound_attackdash", sound_attackdash, Priority::Low)
        .sound_acmd("sound_attackhi3", sound_attackhi3, Priority::Low)
        .sound_acmd("sound_attacks3", sound_attacks3, Priority::Low)
        .sound_acmd("sound_attacks3hi", sound_attacks3hi, Priority::Low)
        .sound_acmd("sound_attacks3lw", sound_attacks3lw, Priority::Low)
        .sound_acmd("sound_specialairs", sound_specialairs, Priority::Low)
        .sound_acmd("sound_specials", sound_specials, Priority::Low)
        .sound_acmd("sound_attacks4", sound_attacks4, Priority::Low)
        .sound_acmd("sound_attackhi4", sound_attackhi4, Priority::Low)
        .install();
    
}