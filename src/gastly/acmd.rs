// src/gastly/acmd.rs - Animation Command Scripts for Gastly

use crate::gastly::FIGHTER_STATES;
use crate::gastly::player_state::EvolutionStage;
use smash::app::lua_bind::*;
use smash::lib::lua_const::*;
use smash::app::utility::get_kind;
use smash::app::BattleObjectModuleAccessor;
use smash::lua2cpp::L2CFighterCommon;
use smash::phx::{Hash40, Vector3f};
use smash::hash40;
use smash::lua2cpp::*;
use smash_script::*;
use smashline::*;

// Import the frame function for ACMD
use smash::app::sv_animcmd::frame;
use smash::app::sv_animcmd::wait;

// Side Special (Ground) Effect
unsafe extern "C" fn effect_specials(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("havel"), 0, 0, 1, 0, 0, 0, 1.5, true);
    }
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("purin_hataku_hold"), Hash40::new("havel"), 0, 0, 1, 0, 0, 0, 1, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
    frame(agent.lua_state_agent, 10.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_hataku"), Hash40::new("purin_hataku"), Hash40::new("top"), 0, 5, 1, 0, 20, 30, 1, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
        macros::LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 3, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, false);
    }
    
    frame(agent.lua_state_agent, 11.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_attack_line"), Hash40::new("top"), 0.0, 3.5, -5.0, 0.0, 0.0, 0.0, 1.5, false);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
        macros::LAST_EFFECT_SET_RATE(agent, 0.5);
    }
    
    frame(agent.lua_state_agent, 13.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_NO_STOP(agent, Hash40::new("purin_hataku_flash"), Hash40::new("havel"), 0, 0, 0.7, 0, 0, 0, 1, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
}

// Side Special (Air) Effect
unsafe extern "C" fn effect_specialairs(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("havel"), 0, 0, 1, 0, 0, 0, 1.5, true);
    }
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("purin_hataku_hold"), Hash40::new("havel"), 0, 0, 1, 0, 0, 0, 1, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
    frame(agent.lua_state_agent, 10.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_hataku"), Hash40::new("purin_hataku"), Hash40::new("top"), 0, 5, 1, 0, 20, 30, 1, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
    
    frame(agent.lua_state_agent, 11.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_attack_line"), Hash40::new("top"), 0.0, 3.5, -5.0, 0.0, 0.0, 0.0, 1.5, false);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
        macros::LAST_EFFECT_SET_RATE(agent, 0.5);
    }
    
    frame(agent.lua_state_agent, 13.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_NO_STOP(agent, Hash40::new("purin_hataku_flash"), Hash40::new("havel"), 0, 0, 0.7, 0, 0, 0, 1, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
}

// Up Special Left Effect
unsafe extern "C" fn effect_specialhil(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        // Custom sys_ripple effect with specified parameters
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 55.0);
    if macros::is_excute(agent) {
        // Second ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 110.0);
    if macros::is_excute(agent) {
        // Third ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
}

// Up Special Right Effect
unsafe extern "C" fn effect_specialhir(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        // Custom sys_ripple effect with specified parameters
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 55.0);
    if macros::is_excute(agent) {
        // Second ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 110.0);
    if macros::is_excute(agent) {
        // Third ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
}

// Up Special Left Effect (Air)
unsafe extern "C" fn effect_specialairhil(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        // Custom sys_ripple effect with specified parameters (air version)
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 55.0);
    if macros::is_excute(agent) {
        // Second ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 110.0);
    if macros::is_excute(agent) {
        // Third ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
}

// Up Special Right Effect (Air)
unsafe extern "C" fn effect_specialairhir(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        // Custom sys_ripple effect with specified parameters (air version)
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 55.0);
    if macros::is_excute(agent) {
        // Second ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
    frame(agent.lua_state_agent, 110.0);
    if macros::is_excute(agent) {
        // Third ripple effect
        macros::EFFECT_FOLLOW(agent, Hash40::new("sys_ripple"), Hash40::new("top"), 0, 6, 0, 90.0, 90.0, 0, 1.0, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.2, 0.5);
        macros::LAST_EFFECT_SET_ALPHA(agent, 2.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.8);
    }
}

// Down Special Left Effect
unsafe extern "C" fn effect_speciallwl(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("purin_nemuru_start"), Hash40::new("top"), 0, 9, 6.5, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, true);
        macros::EFFECT_FOLLOW(agent, Hash40::new("demon_impact"), Hash40::new("top"), 0, 6, 0, 0, 0, 0, 1, true);
    }
    frame(agent.lua_state_agent, 26.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, -6, 0, 0, 0, 1, false);
    }
    frame(agent.lua_state_agent, 40.0);
    for _ in 0..3 {
        if macros::is_excute(agent) {
            macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
        }
        wait(agent.lua_state_agent, 2.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::COL_NORMAL(agent);
        }
        wait(agent.lua_state_agent, 6.0);
    }
    frame(agent.lua_state_agent, 130.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, -6, 0, 0, 0, 1, false);
        macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
    }
    wait(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::COL_NORMAL(agent);
    }
    frame(agent.lua_state_agent, 185.0);
}

// Down Special Right Effect
unsafe extern "C" fn effect_speciallwr(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("purin_nemuru_start"), Hash40::new("top"), 0, 9, 6.5, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, true);
        macros::EFFECT_FOLLOW(agent, Hash40::new("demon_impact"), Hash40::new("top"), 0, 6, 0, 0, 0, 0, 1, true);
    }
    frame(agent.lua_state_agent, 26.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, 6, 0, 0, 0, 1, false);
    }
    frame(agent.lua_state_agent, 40.0);
    for _ in 0..3 {
        if macros::is_excute(agent) {
            macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
        }
        wait(agent.lua_state_agent, 2.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::COL_NORMAL(agent);
        }
        wait(agent.lua_state_agent, 6.0);
    }
    frame(agent.lua_state_agent, 130.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, 6, 0, 0, 0, 1, false);
        macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
    }
    wait(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::COL_NORMAL(agent);
    }
    frame(agent.lua_state_agent, 185.0);
}

// Down Special Left Effect (Air)
unsafe extern "C" fn effect_specialairlwl(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("purin_nemuru_start"), Hash40::new("top"), 0, 9, 6.5, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, true);
        macros::EFFECT_FOLLOW(agent, Hash40::new("demon_impact"), Hash40::new("top"), 0, 6, 0, 0, 0, 0, 1, true);
    }
    frame(agent.lua_state_agent, 26.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, -6, 0, 0, 0, 1, false);
    }
    frame(agent.lua_state_agent, 40.0);
    for _ in 0..3 {
        if macros::is_excute(agent) {
            macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
        }
        wait(agent.lua_state_agent, 2.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::COL_NORMAL(agent);
        }
        wait(agent.lua_state_agent, 6.0);
    }
    frame(agent.lua_state_agent, 130.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, -6, 0, 0, 0, 1, false);
        macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
    }
    wait(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::COL_NORMAL(agent);
    }
    frame(agent.lua_state_agent, 185.0);
}

// Down Special Right Effect (Air)
unsafe extern "C" fn effect_specialairlwr(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("purin_nemuru_start"), Hash40::new("top"), 0, 9, 6.5, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, true);
        macros::EFFECT_FOLLOW(agent, Hash40::new("demon_impact"), Hash40::new("top"), 0, 6, 0, 0, 0, 0, 1, true);
    }
    frame(agent.lua_state_agent, 26.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, 6, 0, 0, 0, 1, false);
    }
    frame(agent.lua_state_agent, 40.0);
    for _ in 0..3 {
        if macros::is_excute(agent) {
            macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
        }
        wait(agent.lua_state_agent, 2.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
        }
        wait(agent.lua_state_agent, 12.0);
        if macros::is_excute(agent) {
            macros::COL_NORMAL(agent);
        }
        wait(agent.lua_state_agent, 6.0);
    }
    frame(agent.lua_state_agent, 130.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_sleep"), Hash40::new("body"), 0, 3, 6, 0, 0, 0, 1, false);
        macros::FLASH(agent, 0.502, 0.314, 0.392, 0.196);
    }
    wait(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.235, 0.549, 0.392);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::FLASH_FRM(agent, 12, 0.941, 0.118, 0.549, 0);
    }
    wait(agent.lua_state_agent, 12.0);
    if macros::is_excute(agent) {
        macros::COL_NORMAL(agent);
    }
    frame(agent.lua_state_agent, 185.0);
}

// Neutral Air Attack Effect
unsafe extern "C" fn effect_attackairn(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_NO_STOP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 3.5, 2.5, 0, 0, 0, 0.65, true);
        macros::LAST_PARTICLE_SET_COLOR(agent, 1, 1, 0.5);
    }
    frame(agent.lua_state_agent, 6.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLIP_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), 2, 3.3, 9, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ, 0.9);
        macros::EFFECT_FOLLOW_NO_STOP(agent, Hash40::new("edge_attack_dash"), Hash40::new("toel"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
}

// Down Tilt Attack Effect
unsafe extern "C" fn effect_attacklw3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 9.0);
    if macros::is_excute(agent) {
        macros::FOOT_EFFECT(agent, Hash40::new("sys_turn_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), 0, 4, 0, -3, 0, 0, 0.6, true, *EF_FLIP_YZ);
        macros::LAST_PARTICLE_SET_COLOR(agent, 1, 1, 0.5);
    }
    frame(agent.lua_state_agent, 10.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLW_POS(agent, Hash40::new("sys_ripstick"), Hash40::new("top"), 0, 4, 5, 0, 0, 0, 0.3, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 1.0, 0.2, 1.0);
        macros::LAST_EFFECT_SET_ALPHA(agent, 1.0);
        macros::LAST_EFFECT_SET_RATE(agent, 0.5);
    }
    frame(agent.lua_state_agent, 11.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLIP_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 4.5, 10.5, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ, 0.6);
    }
}

// Down Taunt effect removal
unsafe extern "C" fn effect_appeallwl(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 21.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("null"), Hash40::new("hip"), 0, 0, 0, -90, -90, 0, 1, true);
    }
    frame(agent.lua_state_agent, 100.0);
    if macros::is_excute(agent) {
        macros::FOOT_EFFECT(agent, Hash40::new("null"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}

unsafe extern "C" fn effect_appeallwr(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 21.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("null"), Hash40::new("hip"), 0, 0, 0, -90, -90, 0, 1, true);
    }
    frame(agent.lua_state_agent, 100.0);
    if macros::is_excute(agent) {
        macros::FOOT_EFFECT(agent, Hash40::new("null"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
    }
}

// Jab 1 - handl for entirety
unsafe extern "C" fn effect_attack11(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::FOOT_EFFECT(agent, Hash40::new("null"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
        macros::EFFECT_FLIP_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), -0.1, 5.5, 12, 0, 0, 0, 0.8, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ, 0.8);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
    }
}

// Jab 2 - handr for entirety
unsafe extern "C" fn effect_attack12(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::FOOT_EFFECT(agent, Hash40::new("null"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.9, 0, 0, 0, 0, 0, 0, false);
        macros::EFFECT_FLIP_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), -2, 4, 9.5, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ, 0.8);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
    }
}

// Forward Tilt - footl for entirety
unsafe extern "C" fn effect_attacks3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 6.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("sys_attack_arc_b"), Hash40::new("top"), 0, 5.5, 3, 0, -25, 190, 0.85, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
        macros::LANDING_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -1, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    
    }
}

// Forward Tilt Up - footl for entirety
unsafe extern "C" fn effect_attacks3hi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
    frame(agent.lua_state_agent, 6.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("sys_attack_arc_b"), Hash40::new("top"), 0, 5.5, 2.5, 20, -25, 160, 0.9, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
    }
}

// Forward Tilt Down - footl for entirety
unsafe extern "C" fn effect_attacks3lw(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -1, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
    frame(agent.lua_state_agent, 6.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc_b"), Hash40::new("sys_attack_arc_b"), Hash40::new("top"), 0, 2, 0, 0, -25, 180, 1, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
    }
}

// Up Tilt - footr for entirety
unsafe extern "C" fn effect_attackhi3(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 7.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_run_smoke"), Hash40::new("top"), -2, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
    frame(agent.lua_state_agent, 9.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_attack_arc_d"), Hash40::new("purin_attack_arc_d"), Hash40::new("top"), -2.5, 10, -2, 0, -130, -103, 0.7, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
        macros::LAST_EFFECT_SET_RATE(agent, 3);
    }
    frame(agent.lua_state_agent, 14.0);
    if macros::is_excute(agent) {
        macros::EFFECT_OFF_KIND(agent, Hash40::new("purin_attack_arc_d"), true, true);
    }
}

// Down Smash - footl and footr for frames 1-35
unsafe extern "C" fn effect_attacklw4(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 9, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
    frame(agent.lua_state_agent, 13.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_smash_line"), Hash40::new("purin_smash_line"), Hash40::new("top"), 1, 1, -2, 0, 0, 0, 0.9, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_RATE(agent, 1.3);
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_smash_line"), Hash40::new("purin_smash_line"), Hash40::new("top"), 1, 1, 2, 0, 180, 0, 0.9, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_RATE(agent, 1.3);
        macros::EFFECT(agent, Hash40::new("edge_gokumon_impact"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
        macros::LAST_EFFECT_SET_RATE(agent, 0.3);
        macros::LAST_EFFECT_SET_COLOR(agent, 1.0, 0.5, 1.0);
        macros::LAST_EFFECT_SET_ALPHA(agent, 1.0);
    }
    frame(agent.lua_state_agent, 14.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLIP(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), 1, 1.5, 11, 0, 0, 0, 1.2, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_NONE);
        macros::EFFECT_FLIP(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), 1, 1.5, -11, 0, 0, 0, 1.2, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_NONE);
    }
    frame(agent.lua_state_agent, 15.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), -5, 0, -2, 0, 0, 0, 0.85, 0, 0, 0, 0, 0, 0, false);
        macros::LAST_EFFECT_SET_ALPHA(agent, 0.7);
        macros::LANDING_EFFECT(agent, Hash40::new("sys_dash_smoke"), Hash40::new("top"), 5, 0, -2, 0, 180, 0, 0.85, 0, 0, 0, 0, 0, 0, false);
        macros::LAST_EFFECT_SET_ALPHA(agent, 0.7);
    }
    frame(agent.lua_state_agent, 36.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_whirlwind_r"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.6, 0, 0, 0, 0, 0, 0, false);
    }
}

// Down Smash Charge - footl and footr for entirety
unsafe extern "C" fn effect_attacklw4charge(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 4.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 1, 14, 4, 14, 0, 0, 0, true);
    }
    frame(agent.lua_state_agent, 8.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 1, 14, 4, 14, 0, 0, 0, true);
    }
    wait(agent.lua_state_agent, 8.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 1, 14, 4, 14, 0, 0, 0, true);
        macros::LANDING_EFFECT(agent, Hash40::new("sys_whirlwind_r"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, true);
        macros::LAST_EFFECT_SET_ALPHA(agent, 0.5);
    }
    wait(agent.lua_state_agent, 8.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash_s"), Hash40::new("top"), 0, 7, 0, 0, 0, 0, 1, 14, 4, 14, 0, 0, 0, true);
    }
    frame(agent.lua_state_agent, 4.0);
}

// Forward Air - footl and footr for frames 1-25
unsafe extern "C" fn effect_attackairf(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 7.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_speedline"), Hash40::new("sys_attack_speedline"), Hash40::new("top"), -1, 4, -3, 0, 0, 0, 1, true, *EF_FLIP_YZ);
        macros::LAST_PARTICLE_SET_COLOR(agent, 1, 1, 0.5);
        macros::EFFECT_FOLLOW_NO_STOP(agent, Hash40::new("edge_attack_dash"), Hash40::new("toel"), 0, 0, 0, 0, 0, 0, 0.5, true);        
    }
    frame(agent.lua_state_agent, 8.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLIP_ALPHA(agent, Hash40::new("sys_attack_impact"), Hash40::new("sys_attack_impact"), Hash40::new("top"), 0, 4.5, 13, 0, 0, 0, 1.1, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ, 0.5);
    }
    frame(agent.lua_state_agent, 25.0);
    if macros::is_excute(agent) {
        macros::EFFECT_OFF_KIND(agent, Hash40::new("edge_attack_dash"), true, true);
    }
}

// Back Air - footr for frames 1-30
unsafe extern "C" fn effect_attackairb(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 11.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 3.7, -3, 0, 190, 165, 0.9, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
        macros::LAST_EFFECT_SET_RATE(agent, 1.6);
    }
}

// Up Air - handl for frames 1-30
unsafe extern "C" fn effect_attackairhi(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 9.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP_ALPHA(agent, Hash40::new("sys_attack_arc"), Hash40::new("sys_attack_arc"), Hash40::new("top"), 0, 8, 1, 0, -110, -90, 0.7, true, *EF_FLIP_YZ, 0.3);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.3, 0.1, 0.5);
        macros::LAST_EFFECT_SET_RATE(agent, 0.9);
    }
}

// Ledge Catch - handl for entirety
unsafe extern "C" fn effect_cliffcatch(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("edge_catch_handaura"), Hash40::new("handl"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_cliff_catch"), Hash40::new("havel"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, false);
    }
}

// NEW FUNCTIONS for attacks that don't have existing effect scripts:

// Dash Grab - handl for entirety
unsafe extern "C" fn effect_catchdash(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("edge_catch_handaura"), Hash40::new("arml"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
    frame(agent.lua_state_agent, 14.0);
    if macros::is_excute(agent) {
        macros::EFFECT_OFF_KIND(agent, Hash40::new("edge_catch_handaura"), true, true);
    }
}

// Pivot Grab - handr for entirety
unsafe extern "C" fn effect_catchturn(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("edge_catch_handaura"), Hash40::new("armr"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
    frame(agent.lua_state_agent, 14.0);
    if macros::is_excute(agent) {
        macros::EFFECT_OFF_KIND(agent, Hash40::new("edge_catch_handaura"), true, true);
    }
}

// Standing Grab - handl for entirety
unsafe extern "C" fn effect_catch(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 5.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("edge_catch_handaura"), Hash40::new("arml"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
    frame(agent.lua_state_agent, 14.0);
    if macros::is_excute(agent) {
        macros::EFFECT_OFF_KIND(agent, Hash40::new("edge_catch_handaura"), true, true);
    }
}

// Ledge Wait - handl for entirety
unsafe extern "C" fn effect_cliffwait(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 1.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("edge_catch_handaura"), Hash40::new("arml"), 0, 0, 0, 0, 0, 0, 1.0, true);
    }
}

// Up Smash
unsafe extern "C" fn effect_attackhi4(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 3.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 11, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
    frame(agent.lua_state_agent, 15.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW_FLIP(agent, Hash40::new("purin_smash_arc"), Hash40::new("purin_smash_arc"), Hash40::new("top"), -2, 6.2, -3, 180, -90, 90, 1, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
        macros::LAST_EFFECT_SET_RATE(agent, 1.3);
    }
    frame(agent.lua_state_agent, 20.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_down_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
}

// Forward Smash
unsafe extern "C" fn effect_attacks4(agent: &mut L2CAgentBase) {
    frame(agent.lua_state_agent, 2.0);
    if macros::is_excute(agent) {
        macros::EFFECT(agent, Hash40::new("sys_smash_flash"), Hash40::new("top"), 0, 2.5, 8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
    }
    frame(agent.lua_state_agent, 11.0);
    if macros::is_excute(agent) {
        macros::LANDING_EFFECT(agent, Hash40::new("sys_atk_smoke"), Hash40::new("top"), 0, 0, 0, 0, 0, 0, 0.7, 0, 0, 0, 0, 0, 0, false);
    }
    frame(agent.lua_state_agent, 16.0);
    if macros::is_excute(agent) {
        macros::EFFECT_FLIP(agent, Hash40::new("purin_smash_line"), Hash40::new("purin_smash_line"), Hash40::new("top"), 0, 6.7, -8.5, 0, 0, 0, 1.2, 0, 0, 0, 0, 0, 0, true, *EF_FLIP_YZ);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.43);
    }
}

unsafe extern "C" fn effect_catchpull(agent: &mut L2CAgentBase) {
    // Check if this player is in Gastly stage
    let boma = agent.module_accessor;
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let is_gastly_stage = {
        let states_map = FIGHTER_STATES.read();
        states_map.get(&entry_id)
            .map(|state| state.stage == EvolutionStage::Gastly)
            .unwrap_or(false)
    };
    
    if is_gastly_stage {
        if macros::is_excute(agent) {
            macros::EFFECT(agent, Hash40::new("ridley_grabbing_catch"), Hash40::new("top"), 0, 3, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
        }
    } else {
        //  Stop g_grab_burn sound for non-Gastly stages
        SoundModule::stop_se(boma, Hash40::new("g_grab_burn"), 0);
    }
}

unsafe extern "C" fn effect_catchwait(agent: &mut L2CAgentBase) {
    // Check if this player is in Gastly stage
    let boma = agent.module_accessor;
    let entry_id = WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
    
    let is_gastly_stage = {
        let states_map = FIGHTER_STATES.read();
        states_map.get(&entry_id)
            .map(|state| state.stage == EvolutionStage::Gastly)
            .unwrap_or(false)
    };
    
    if is_gastly_stage {
        if macros::is_excute(agent) {
            macros::EFFECT(agent, Hash40::new("ridley_grabbing_catch"), Hash40::new("top"), 0, 3, 10, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, true);
        }
    } else {
        //  Stop g_grab_burn sound for non-Gastly stages
        SoundModule::stop_se(boma, Hash40::new("g_grab_burn"), 0);
    }
}

// Neutral Special Rollout Wind Effect
unsafe extern "C" fn effect_specialnkorogaruwind(agent: &mut L2CAgentBase) {
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("purin_korogaru_wind"), Hash40::new("body"), 0, 0, 0, 0, 0, 0, 1, true);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.4);
        EffectModule::enable_sync_init_pos_last(agent.module_accessor);
    }
}

// Neutral Special Turn Effect (Ground)
unsafe extern "C" fn effect_specialnturneffect(agent: &mut L2CAgentBase) {
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("purin_korogaru_loop"), Hash40::new("body"), 0, 0, 0, 0, 0, 0, 1, false);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.4);
    }
}

// Neutral Special Turn Effect (Air)
unsafe extern "C" fn effect_specialairnturneffect(agent: &mut L2CAgentBase) {
    if macros::is_excute(agent) {
        macros::EFFECT_FOLLOW(agent, Hash40::new("purin_korogaru_loop"), Hash40::new("body"), 0, 0, 0, 0, 0, 0, 1, false);
        macros::LAST_EFFECT_SET_COLOR(agent, 0.2, 0.1, 0.4);
    }
}


pub fn install_acmd_with_costumes(costume: &[usize]) {
    Agent::new("purin")
        .set_costume(costume.to_vec())
        .effect_acmd("effect_specials", effect_specials, Priority::Low)
        .effect_acmd("effect_specialairs", effect_specialairs, Priority::Low)
        .effect_acmd("effect_specialhil", effect_specialhil, Priority::Low)
        .effect_acmd("effect_specialhir", effect_specialhir, Priority::Low)
        .effect_acmd("effect_specialairhil", effect_specialairhil, Priority::Low)
        .effect_acmd("effect_specialairhir", effect_specialairhir, Priority::Low)
        .effect_acmd("effect_speciallwl", effect_speciallwl, Priority::Low)
        .effect_acmd("effect_speciallwr", effect_speciallwr, Priority::Low)
        .effect_acmd("effect_specialairlwl", effect_speciallwl, Priority::Low)
        .effect_acmd("effect_specialairlwr", effect_speciallwr, Priority::Low)
        .effect_acmd("effect_attackairn", effect_attackairn, Priority::Low)
        .effect_acmd("effect_attacklw3", effect_attacklw3, Priority::Low)
        .effect_acmd("effect_appeallwl", effect_appeallwl, Priority::Low)
        .effect_acmd("effect_appeallwr", effect_appeallwr, Priority::Low)
        .effect_acmd("effect_attack11", effect_attack11, Priority::Low)
        .effect_acmd("effect_attack12", effect_attack12, Priority::Low)
        .effect_acmd("effect_attacks3", effect_attacks3, Priority::Low)
        .effect_acmd("effect_attacks3hi", effect_attacks3hi, Priority::Low)
        .effect_acmd("effect_attacks3lw", effect_attacks3lw, Priority::Low)
        .effect_acmd("effect_attackhi3", effect_attackhi3, Priority::Low)
        .effect_acmd("effect_attacklw4", effect_attacklw4, Priority::Low)
        .effect_acmd("effect_attacklw4charge", effect_attacklw4charge, Priority::Low)
        .effect_acmd("effect_attackairf", effect_attackairf, Priority::Low)
        .effect_acmd("effect_attackairb", effect_attackairb, Priority::Low)
        .effect_acmd("effect_attackairhi", effect_attackairhi, Priority::Low)
        .effect_acmd("effect_cliffcatch", effect_cliffcatch, Priority::Low)
        .effect_acmd("effect_catchdash", effect_catchdash, Priority::Low)
        .effect_acmd("effect_catchturn", effect_catchturn, Priority::Low)
        .effect_acmd("effect_catch", effect_catch, Priority::Low)
        .effect_acmd("effect_cliffwait", effect_cliffwait, Priority::Low)
        .effect_acmd("effect_attacks4", effect_attacks4, Priority::Low)
        .effect_acmd("effect_attackhi4", effect_attackhi4, Priority::Low)
        .effect_acmd("effect_catchpull", effect_catchpull, Priority::Low)
        .effect_acmd("effect_catchwait", effect_catchwait, Priority::Low)
        .effect_acmd("effect_specialnkorogaruwind", effect_specialnkorogaruwind, Priority::Low)
        .effect_acmd("effect_specialnturneffect", effect_specialnturneffect, Priority::Low)
        .effect_acmd("effect_specialairnturneffect", effect_specialairnturneffect, Priority::Low)
        .install();
    
    }