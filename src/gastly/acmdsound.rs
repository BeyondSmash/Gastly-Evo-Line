// src/gastly/acmdsound.rs - Vanilla ACMD overrides with custom sound additions

use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::app::BattleObjectModuleAccessor;
use smash::phx::Hash40;
use smash::lua2cpp::L2CFighterCommon;
use smash_script::macros;
use smashline::*;
use smash::app::sv_animcmd::*;
use smash_script::sound;
use smash_script::lua_args;

// Import our sound system
use crate::gastly::{FIGHTER_STATES};
use crate::gastly::player_state::EvolutionStage;

unsafe fn get_attack_voice_correct(boma: *mut BattleObjectModuleAccessor, attack_type: &str) -> &'static str {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let states_map = FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&entry_id) {
        match player_state.stage {
            crate::gastly::player_state::EvolutionStage::Gastly => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "gas_attack01",
                    2 => "gas_attack02_07",
                    3 => "gas_attack03",
                    4 => "gas_attack06",
                    _ => "gas_attack01",
                }
            },
            crate::gastly::player_state::EvolutionStage::Haunter => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "hau_attack01",
                    2 => "hau_attack02_07",
                    3 => "hau_attack03",
                    4 => "hau_attack06",
                    _ => "hau_attack01",
                }
            },
            crate::gastly::player_state::EvolutionStage::Gengar => {
                let random = crate::gastly::random_module::rand_range_i32(1, 4);
                match random {
                    1 => "gen_attack01",
                    2 => "gen_attack02_07",
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

// Up Taunt Left - Stage-specific appeal voice
unsafe extern "C" fn sound_appealhil(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        //  Custom sound based on evolution stage
        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
        let appeal_sound = {
            let states_map = FIGHTER_STATES.read();
            states_map.get(&entry_id)
                .map(|state| match state.stage {
                    crate::gastly::player_state::EvolutionStage::Gastly => "gas_appeal01",
                    crate::gastly::player_state::EvolutionStage::Haunter => "hau_appeal01", 
                    crate::gastly::player_state::EvolutionStage::Gengar => "gen_appeal01",
                })
                .unwrap_or("gas_appeal01") // Default to Gastly
        };
        
        // Play stage-specific appeal voice
        let appeal_handle = SoundModule::play_se(boma, Hash40::new(appeal_sound), true, false, false, false, smash::app::enSEType(0));
        SoundModule::set_se_vol(agent.module_accessor, appeal_handle as i32, 1.5, 0);
        
        // Keep existing custom sound
        let appealh = SoundModule::play_se(boma, Hash40::new("g_appeal_h01"), true, false, false, false, smash::app::enSEType(0));
        SoundModule::set_se_vol(agent.module_accessor, appealh as i32, 0.5, 0);
        
        // VANILLA: Original sounds (but skip the vanilla voice)
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 25.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 8.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump02_02"));
    }
    wait(lua_state, 43.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_landing01"));
    }
}

// Up Taunt Right - Stage-specific appeal voice
unsafe extern "C" fn sound_appealhir(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        //  Custom sound based on evolution stage
        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
        let appeal_sound = {
            let states_map = FIGHTER_STATES.read();
            states_map.get(&entry_id)
                .map(|state| match state.stage {
                    crate::gastly::player_state::EvolutionStage::Gastly => "gas_appeal01",
                    crate::gastly::player_state::EvolutionStage::Haunter => "hau_appeal01",
                    crate::gastly::player_state::EvolutionStage::Gengar => "gen_appeal01",
                })
                .unwrap_or("gas_appeal01") // Default to Gastly
        };
        
        // Play stage-specific appeal voice
        let appeal_handle = SoundModule::play_se(boma, Hash40::new(appeal_sound), true, false, false, false, smash::app::enSEType(0));
        SoundModule::set_se_vol(agent.module_accessor, appeal_handle as i32, 1.5, 0);
        
        // Keep existing custom sound
        let appealh2 = SoundModule::play_se(boma, Hash40::new("g_appeal_h01"), true, false, false, false, smash::app::enSEType(0));
        SoundModule::set_se_vol(agent.module_accessor, appealh2 as i32, 0.5, 0);
        
        // VANILLA: Original sounds (but skip the vanilla voice)
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 25.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 8.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump02_02"));
    }
    wait(lua_state, 43.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_landing01"));
    }
}

// Down Air - Custom for Gastly/Haunter, Vanilla for Gengar
unsafe extern "C" fn sound_attackairlw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let is_gastly_or_haunter = {
        let states_map = FIGHTER_STATES.read();
        states_map.get(&entry_id)
            .map(|state| matches!(state.stage, 
                crate::gastly::player_state::EvolutionStage::Gastly | 
                crate::gastly::player_state::EvolutionStage::Haunter))
            .unwrap_or(true) // Default to custom sound
    };
    
    if is_gastly_or_haunter {
        // Custom sound for Gastly and Haunter
        frame(lua_state, 5.0);
        if macros::is_excute(agent) {
            let dair_tongue = SoundModule::play_se(boma, Hash40::new("gas_hau_tongue_dair"), true, false, false, false, smash::app::enSEType(0));
            SoundModule::set_se_vol(agent.module_accessor, dair_tongue as i32, 1.3, 0);
        }
    } else {
        // Vanilla Jigglypuff down air for Gengar
        frame(lua_state, 5.0);
        if macros::is_excute(agent) {
            macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack01"));
            macros::PLAY_SE(agent, Hash40::new("se_purin_swing_m"));
            macros::PLAY_SE(agent, Hash40::new("se_common_swing_02"));
        }
        wait(lua_state, 6.0);
        if macros::is_excute(agent) {
            macros::PLAY_SE(agent, Hash40::new("se_purin_swing_m"));
            macros::PLAY_SE(agent, Hash40::new("se_common_swing_02"));
        }
        wait(lua_state, 6.0);
        if macros::is_excute(agent) {
            macros::PLAY_SE(agent, Hash40::new("se_purin_swing_m"));
            macros::PLAY_SE(agent, Hash40::new("se_common_swing_02"));
        }
        wait(lua_state, 6.0);
        if macros::is_excute(agent) {
            macros::PLAY_SE(agent, Hash40::new("se_purin_swing_m"));
            macros::PLAY_SE(agent, Hash40::new("se_common_swing_02"));
        }
    }
}

// Down Smash - Converted with 20% attack voice chance
unsafe extern "C" fn sound_attacklw4(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 14.0);
    if macros::is_excute(agent) {
        // Custom sound
        let dsmash = SoundModule::play_se(
                boma,
                Hash40::new("g_down_smash"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, dsmash as i32, 0.5, 0);
        
        // VANILLA: Original sounds
        macros::STOP_SE(agent, Hash40::new("se_common_smash_start_04"));
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack_smash_l"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_ll"));
    }
    
    //  Add attack voice 1 frame after PLAY_SEQUENCE
    frame(lua_state, 15.0);
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
    
    wait(lua_state, 28.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_step_left_m"));
    }
    wait(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_step_right_m"));
    }
}

// Down Tilt - Updated with 20% chance and correct file names
unsafe extern "C" fn sound_attacklw3(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 5.0);
    if macros::is_excute(agent) {
        let gdtilt = SoundModule::play_se(
                boma,
                Hash40::new("g_down_tilt"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, gdtilt as i32, 0.3, 0);
    }
    
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

// Forward Air - Updated with 20% chance and correct file names
unsafe extern "C" fn sound_attackairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 1.0);
    if macros::is_excute(agent) {
        let fair_handle = SoundModule::play_se(
            boma,
            Hash40::new("g_nair"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, fair_handle as i32, 0.2, 0);
    }
    
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

// Neutral Air - Updated with 20% chance and correct file names
unsafe extern "C" fn sound_attackairn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        let nair_handle = SoundModule::play_se(
            boma,
            Hash40::new("g_nair"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, nair_handle as i32, 0.3, 0);
    }
    
    frame(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_attack01"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_swing_l"));
    }
    
    frame(lua_state, 7.0);
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

// Shield Break Fly - Add g_shieldbreak on frame 2
unsafe extern "C" fn sound_shieldbreakfly(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_guardbreak"));
        macros::PLAY_SE(agent, Hash40::new("g_shieldbreak"));
        macros::PLAY_SEQUENCE(agent, Hash40::new("seq_purin_rnd_futtobi01"));
    }
}

// Down Taunt Left - Add custom sounds + landing logic
unsafe extern "C" fn sound_appeallwl(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 1.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("f01_appeal_lw"));
    }
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump01"));
    }
    wait(lua_state, 13.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_purin_jump01"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_appeal_l01"));
    }
    wait(lua_state, 10.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appeal_l02"));
    }
    frame(lua_state, 80.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump02_02"));
    }
    frame(lua_state, 90.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("f90_appeal_lw"));
        let landing_sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_SE(agent, Hash40::new(landing_sound));
    }
}

// Down Taunt Right - Add custom sounds + landing logic
unsafe extern "C" fn sound_appeallwr(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 1.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("f01_appeal_lw"));
    }
    
    frame(lua_state, 11.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump01"));
    }
    wait(lua_state, 13.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_purin_jump01"));
        macros::PLAY_SE(agent, Hash40::new("se_purin_appeal_l01"));
    }
    wait(lua_state, 10.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appeal_l02"));
    }
    frame(lua_state, 80.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_jump02_02"));
    }
    frame(lua_state, 90.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("f90_appeal_lw"));
        let landing_sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_SE(agent, Hash40::new(landing_sound));
    }
}

// Landing functions - gas_hau_landing for Gastly/Haunter only
unsafe fn get_landing_sound(boma: *mut BattleObjectModuleAccessor, fallback: &'static str) -> &'static str {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let states_map = FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&entry_id) {
        match player_state.stage {
            crate::gastly::player_state::EvolutionStage::Gastly | 
            crate::gastly::player_state::EvolutionStage::Haunter => "gas_hau_landing",
            crate::gastly::player_state::EvolutionStage::Gengar => fallback,
        }
    } else {
        "gas_hau_landing" // Default to Gastly
    }
}

unsafe extern "C" fn sound_landingairb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
}
    }
}

unsafe extern "C" fn sound_landingairf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_landingairhi(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_landingairlw(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_landingairn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_landingfallspecial(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_special_l01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

unsafe extern "C" fn sound_landingheavy(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 4.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_landinglight(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
        let states_map = FIGHTER_STATES.read();
        
        if let Some(player_state) = states_map.get(&entry_id) {
            match player_state.stage {
                crate::gastly::player_state::EvolutionStage::Gastly | 
                crate::gastly::player_state::EvolutionStage::Haunter => {
                    let landing_handle = SoundModule::play_se(
                        boma,
                        Hash40::new("gas_hau_landing"),
                        false, false, false, false,
                        smash::app::enSEType(0)
                    );
                    SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
                }
                crate::gastly::player_state::EvolutionStage::Gengar => {
                    macros::PLAY_LANDING_SE(agent, Hash40::new("se_purin_landing01"));
                }
            }
        } else {
            // Default to Gastly
            let landing_handle = SoundModule::play_se(
                boma,
                Hash40::new("gas_hau_landing"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
        }
    }
}

unsafe extern "C" fn sound_stepposeback(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 2.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing02");
    if sound == "gas_hau_landing" {
        let landing_handle = SoundModule::play_se(
            boma,
            Hash40::new(sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    } else {
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
    }
}

unsafe extern "C" fn sound_passive(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_s"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_m"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_l"));
        macros::PLAY_SE(agent, Hash40::new("vc_purin_passive"));
        let sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

unsafe extern "C" fn sound_passiveceil(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_s"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_m"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_l"));
        macros::PLAY_SE(agent, Hash40::new("vc_purin_passive"));
        let sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

unsafe extern "C" fn sound_passivestandb(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_escape"));
    }
    frame(lua_state, 23.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

unsafe extern "C" fn sound_passivestandf(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_escape"));
    }
    frame(lua_state, 23.0);
    if macros::is_excute(agent) {
        let sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

unsafe extern "C" fn sound_passivewall(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_s"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_m"));
        macros::STOP_SE(agent, Hash40::new("se_common_blowaway_l"));
        macros::PLAY_SE(agent, Hash40::new("vc_purin_passive"));
        let sound = get_landing_sound(boma, "se_purin_landing01");
        macros::PLAY_LANDING_SE(agent, Hash40::new(sound));
    }
}

// Helper function to check if current stage should mute step sounds
unsafe fn should_mute_steps(boma: *mut BattleObjectModuleAccessor) -> bool {
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let states_map = FIGHTER_STATES.read();
    if let Some(player_state) = states_map.get(&entry_id) {
        matches!(player_state.stage, 
                crate::gastly::player_state::EvolutionStage::Gastly | 
                crate::gastly::player_state::EvolutionStage::Haunter)
    } else {
        true // Default to Gastly (mute steps)
    }
}

// Turn - Mute steps for Gastly/Haunter only
unsafe extern "C" fn sound_turn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    if !should_mute_steps(boma) {
        // Keep vanilla sounds for Gengar
        frame(lua_state, 6.0);
        if macros::is_excute(agent) {
            macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_m"));
        }
        wait(lua_state, 5.0);
        if macros::is_excute(agent) {
            macros::PLAY_STEP(agent, Hash40::new("se_purin_step_left_m"));
        }
    }
    // Gastly/Haunter get no step sounds
}

// Turn Dash - Mute steps for Gastly/Haunter only
unsafe extern "C" fn sound_turndash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 8.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_dash_start"));
        macros::SET_PLAY_INHIVIT(agent, Hash40::new("se_purin_dash_start"), 20);
    }
    
    if !should_mute_steps(boma) {
        // Keep vanilla step sounds for Gengar only
        wait(lua_state, 6.0);
        if macros::is_excute(agent) {
            macros::PLAY_STEP(agent, Hash40::new("se_purin_step_right_m"));
        }
        wait(lua_state, 5.0);
        if macros::is_excute(agent) {
            macros::PLAY_STEP(agent, Hash40::new("se_purin_step_left_m"));
        }
    }
}

// Up Special Left Ground - Add g_hypnosis on frame 3
unsafe extern "C" fn sound_specialhil(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        let hypno = SoundModule::play_se(
                boma,
                Hash40::new("g_hypnosis"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, hypno as i32, 0.5, 0);
    }
    
    frame(lua_state, 27.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_003"));
    }
    wait(lua_state, 38.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_004"));
    }
}

// Up Special Right Ground - Add g_hypnosis on frame 3
unsafe extern "C" fn sound_specialhir(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        let hypno = SoundModule::play_se(
                boma,
                Hash40::new("g_hypnosis"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, hypno as i32, 0.5, 0);
    }
    
    frame(lua_state, 27.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_003"));
    }
    wait(lua_state, 38.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_004"));
    }
}

// Up Special Left Air - Add g_hypnosis on frame 3
unsafe extern "C" fn sound_specialairhil(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        let hypno = SoundModule::play_se(
                boma,
                Hash40::new("g_hypnosis"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, hypno as i32, 0.5, 0);
    }
    
    frame(lua_state, 27.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_003"));
    }
    wait(lua_state, 38.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_004"));
    }
}

// Up Special Right Air - Add g_hypnosis on frame 3
unsafe extern "C" fn sound_specialairhir(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 3.0);
    if macros::is_excute(agent) {
        let hypno = SoundModule::play_se(
                boma,
                Hash40::new("g_hypnosis"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, hypno as i32, 0.5, 0);
    }
    
    frame(lua_state, 27.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_003"));
    }
    wait(lua_state, 38.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_004"));
    }
}

// Entry Left - Add cry_gastly on frame 65 and replace landing sounds
unsafe extern "C" fn sound_entryl(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 19.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear02"));
    }
    wait(lua_state, 2.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear03"));
    }
    wait(lua_state, 44.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    
    // Frame 62: Replace landing sound with gas_hau_landing
    frame(lua_state, 62.0);
    if macros::is_excute(agent) {
        let landing_handle = SoundModule::play_se(
                boma,
                Hash40::new("gas_hau_landing"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    }
    
    // Frame 65: Add cry_gastly
    frame(lua_state, 65.0);
    if macros::is_excute(agent) {
        let cry_handle = SoundModule::play_se(
            boma,
            Hash40::new("cry_gastly"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, cry_handle as i32, 2.0, 0);
    }
    
    // Frame 95: Replace landing sound with gas_hau_landing
    frame(lua_state, 95.0);
    if macros::is_excute(agent) {
        let landing_handle = SoundModule::play_se(
                boma,
                Hash40::new("gas_hau_landing"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    }
}

// Entry Right - Add cry_gastly on frame 65 and replace landing sounds
unsafe extern "C" fn sound_entryr(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 19.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear02"));
    }
    wait(lua_state, 2.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear03"));
    }
    wait(lua_state, 44.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_appear01"));
    }
    
    // Frame 62: Replace landing sound with gas_hau_landing
    frame(lua_state, 62.0);
    if macros::is_excute(agent) {
        let landing_handle = SoundModule::play_se(
                boma,
                Hash40::new("gas_hau_landing"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    }
    
    // Frame 65: Add cry_gastly
    frame(lua_state, 65.0);
    if macros::is_excute(agent) {
        let cry_handle = SoundModule::play_se(
            boma,
            Hash40::new("cry_gastly"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, cry_handle as i32, 2.0, 0);
    }
    
    // Frame 95: Replace landing sound with gas_hau_landing
    frame(lua_state, 95.0);
    if macros::is_excute(agent) {
        let landing_handle = SoundModule::play_se(
                boma,
                Hash40::new("gas_hau_landing"),
                false, false, false, false,
                smash::app::enSEType(0)
            );
            SoundModule::set_se_vol(boma, landing_handle as i32, 0.3, 0);
    }
}

unsafe extern "C" fn sound_catch(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    frame(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 6.0);
    if macros::is_excute(agent) {
        sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
    }
}

unsafe extern "C" fn sound_catchpull(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    frame(lua_state, 1.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new(""));
    }
    wait(lua_state, 2.0);
    if macros::is_excute(agent) {
        sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
    }
}

unsafe extern "C" fn sound_catchdash(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    // Get current evolution stage
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    let current_stage = {
        let states_map = FIGHTER_STATES.read();
        states_map.get(&entry_id)
            .map(|state| state.stage)
            .unwrap_or(EvolutionStage::Gastly)
    };
    
    frame(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 6.0);
    if macros::is_excute(agent) {
        sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
    }
    wait(lua_state, 1.0);
    if macros::is_excute(agent) {
        // Only play landing sound for Gengar
        if current_stage == EvolutionStage::Gengar {
            macros::PLAY_LANDING_SE(agent, Hash40::new("se_purin_landing01"));
        }
    }
    wait(lua_state, 10.0);
    if macros::is_excute(agent) {
        // Only play landing sound for Gengar
        if current_stage == EvolutionStage::Gengar {
            macros::PLAY_LANDING_SE(agent, Hash40::new("se_purin_landing01"));
        }
    }
    wait(lua_state, 13.0);
    if macros::is_excute(agent) {
        // Only play landing sound for Gengar
        if current_stage == EvolutionStage::Gengar {
            macros::PLAY_LANDING_SE(agent, Hash40::new("se_purin_landing02"));
        }
    }
}

unsafe extern "C" fn sound_catchturn(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    frame(lua_state, 6.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_appear01"));
    }
    wait(lua_state, 6.0);
    if macros::is_excute(agent) {
        sound!(agent, *MA_MSC_CMD_SOUND_STOP_SE_STATUS);
    }
}

const GASTLY_FINALSHOUT_VOLUME: f32 = 3.7;
const HAUNTER_FINALSHOUT_VOLUME: f32 = 3.5;
const GENGAR_FINALSHOUT_VOLUME: f32 = 3.5;
const MEGA_FINALSHOUT_VOLUME: f32 = 2.1;
const GIGA_FINALSHOUT_VOLUME: f32 = 2.5;

unsafe extern "C" fn sound_final(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 10.0);
    if macros::is_excute(agent) {
        // Get current evolution stage and final smash form
        let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
        let (final_sound, volume) = {
            let states_map = FIGHTER_STATES.read();
            if let Some(player_state) = states_map.get(&entry_id) {
                // Check for final smash forms first
                if player_state.is_in_final_smash_form {
                    if player_state.mega_gengar_form_active {
                        ("mega_finalshout", MEGA_FINALSHOUT_VOLUME)
                    } else if player_state.giga_gengar_form_active {
                        ("giga_finalshout", GIGA_FINALSHOUT_VOLUME)
                    } else {
                        // Fallback to regular stage sound
                        match player_state.stage {
                            EvolutionStage::Gastly => ("gastly_finalshout", GASTLY_FINALSHOUT_VOLUME),
                            EvolutionStage::Haunter => ("haunter_finalshout", HAUNTER_FINALSHOUT_VOLUME),
                            EvolutionStage::Gengar => ("gengar_finalshout", GENGAR_FINALSHOUT_VOLUME),
                        }
                    }
                } else {
                    // Regular evolution stages
                    match player_state.stage {
                        EvolutionStage::Gastly => ("gastly_finalshout", GASTLY_FINALSHOUT_VOLUME),
                        EvolutionStage::Haunter => ("haunter_finalshout", HAUNTER_FINALSHOUT_VOLUME),
                        EvolutionStage::Gengar => ("gengar_finalshout", GENGAR_FINALSHOUT_VOLUME),
                    }
                }
            } else {
                ("gastly_finalshout", GASTLY_FINALSHOUT_VOLUME) // Default
            }
        };
        
        let final_handle = SoundModule::play_se(
            boma,
            Hash40::new(final_sound),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, final_handle as i32, volume, 0);
    }
}

unsafe extern "C" fn sound_speciallwl(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 34.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_001"));
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_sleep"));
    }
    wait(lua_state, 100.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_sleep"));
    }
    wait(lua_state, 53.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_purin_sleep"));
        
        // Use SoundModule::play_se with volume control for vc_purin_002
        let voice_handle = SoundModule::play_se(
            boma,
            Hash40::new("vc_purin_002"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, voice_handle as i32, 6.0, 0);
    }
    wait(lua_state, 24.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_l01"));
    }
    wait(lua_state, 12.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_l01"));
    }
}

unsafe extern "C" fn sound_speciallwr(agent: &mut L2CAgentBase) {
    let lua_state = agent.lua_state_agent;
    let boma = agent.module_accessor;
    
    frame(lua_state, 34.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("vc_purin_001"));
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_sleep"));
    }
    wait(lua_state, 100.0);
    if macros::is_excute(agent) {
        macros::PLAY_STATUS(agent, Hash40::new("se_purin_sleep"));
    }
    wait(lua_state, 53.0);
    if macros::is_excute(agent) {
        macros::STOP_SE(agent, Hash40::new("se_purin_sleep"));
        
        // Use SoundModule::play_se with volume control for vc_purin_002
        let voice_handle = SoundModule::play_se(
            boma,
            Hash40::new("vc_purin_002"),
            false, false, false, false,
            smash::app::enSEType(0)
        );
        SoundModule::set_se_vol(boma, voice_handle as i32, 6.0, 0);
    }
    wait(lua_state, 24.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_l01"));
    }
    wait(lua_state, 12.0);
    if macros::is_excute(agent) {
        macros::PLAY_SE(agent, Hash40::new("se_purin_special_l01"));
    }
}

// Install ACMD sound overrides
pub fn install_acmd_sound_with_costumes(costume: &[usize]) {
    Agent::new("purin")
        .set_costume(costume.to_vec())
        .sound_acmd("sound_appealhil", sound_appealhil, Priority::Low)
        .sound_acmd("sound_appealhir", sound_appealhir, Priority::Low)
        .sound_acmd("sound_attackairlw", sound_attackairlw, Priority::Low)
        .sound_acmd("sound_attacklw4", sound_attacklw4, Priority::Low)
        .sound_acmd("sound_attacklw3", sound_attacklw3, Priority::Low)
        .sound_acmd("sound_attackairf", sound_attackairf, Priority::Low)
        .sound_acmd("sound_attackairn", sound_attackairn, Priority::Low)
        .sound_acmd("sound_shieldbreakfly", sound_shieldbreakfly, Priority::Low)
        .sound_acmd("sound_appeallwl", sound_appeallwl, Priority::Low)
        .sound_acmd("sound_appeallwr", sound_appeallwr, Priority::Low)
        .sound_acmd("sound_landingairb", sound_landingairb, Priority::Low)
        .sound_acmd("sound_landingairf", sound_landingairf, Priority::Low)
        .sound_acmd("sound_landingairhi", sound_landingairhi, Priority::Low)
        .sound_acmd("sound_landingairlw", sound_landingairlw, Priority::Low)
        .sound_acmd("sound_landingairn", sound_landingairn, Priority::Low)
        .sound_acmd("sound_landingfallspecial", sound_landingfallspecial, Priority::Low)
        .sound_acmd("sound_landingheavy", sound_landingheavy, Priority::Low)
        .sound_acmd("sound_landinglight", sound_landinglight, Priority::Low)
        .sound_acmd("sound_stepposeback", sound_stepposeback, Priority::Low)
        .sound_acmd("sound_passive", sound_passive, Priority::Low)
        .sound_acmd("sound_passiveceil", sound_passiveceil, Priority::Low)
        .sound_acmd("sound_passivestandb", sound_passivestandb, Priority::Low)
        .sound_acmd("sound_passivestandf", sound_passivestandf, Priority::Low)
        .sound_acmd("sound_passivewall", sound_passivewall, Priority::Low)
        .sound_acmd("sound_turn", sound_turn, Priority::Low)
        .sound_acmd("sound_turndash", sound_turndash, Priority::Low)
        .sound_acmd("sound_specialhil", sound_specialhil, Priority::Low)
        .sound_acmd("sound_specialhir", sound_specialhir, Priority::Low)
        .sound_acmd("sound_specialairhil", sound_specialairhil, Priority::Low)
        .sound_acmd("sound_specialairhir", sound_specialairhir, Priority::Low)
        .sound_acmd("sound_entryl", sound_entryl, Priority::Low)
        .sound_acmd("sound_entryr", sound_entryr, Priority::Low)
        .sound_acmd("sound_catch", sound_catch, Priority::Low)
        .sound_acmd("sound_catchpull", sound_catchpull, Priority::Low)
        .sound_acmd("sound_catchdash", sound_catchdash, Priority::Low)
        .sound_acmd("sound_catchturn", sound_catchturn, Priority::Low)
        .sound_acmd("sound_final", sound_final, Priority::Low)
        .sound_acmd("sound_speciallwl", sound_speciallwl, Priority::Low)
        .sound_acmd("sound_speciallwr", sound_speciallwr, Priority::Low)
        .install();
    
}