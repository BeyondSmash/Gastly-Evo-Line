// src/gastly/player_state.rs - UPDATED TO REMOVE OLD AURA TRACKING

use std::collections::HashMap;
use smash::app::BattleObjectModuleAccessor;
use smash::app::lua_bind::{WorkModule, DamageModule, ModelModule, VisibilityModule, AttackModule, StatusModule};
use smash::lua2cpp::L2CFighterCommon;
use smash_script::macros;
use smash::lib::lua_const::*;
use smash::phx::{Hash40, Vector3f};
use smash::app::utility;
use skyline::libc::c_uint;

// Import from our modules
use crate::gastly::constants::*;
use crate::gastly::visuals::{update_body_and_unique_parts_visibility, set_active_eye_mesh, handle_final_smash_model_swap};
use crate::gastly::icon_management::{enforce_icon_exclusivity, update_is_any_pos_sensitive_icon_active_flag};
use crate::gastly::random_module;
use crate::gastly::effects::kill_gastly_aura_on_evolution;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStage { Gastly, Haunter, Gengar }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlinkPhase { Open, HalfClose, FullClose, HalfOpen }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitboxDetectionMethod {
    None,
    AttackModuleInfliction,
    AttackModuleOccur,
    AttackModuleSpecificId(i32),
    FrameBased,
}

#[derive(Debug, Clone)]
pub struct PlayerEvolutionState {
    pub stage: EvolutionStage,
    pub blink_timer: i32,
    pub blink_phase: BlinkPhase,
    pub damage_received_this_stage: f32,
    pub hits_landed_this_stage: i32,
    pub previous_total_damage: f32,

    pub shadowball_status_frames: i32,
    pub is_in_shadowball_status: bool,
    pub last_shadowball_status: i32,

    pub last_status_during_evolution: i32,

    pub shadowball_was_sufficiently_charged: bool,
    pub shadowball_air_charge_count: i32,
    pub shadowball_previous_status: i32,

    pub opponent_damage_tracking: HashMap<u32, f32>,
    pub was_attacking_last_frame: bool,

    pub vanilla_expression_tracking: bool,
    pub last_vanilla_expression: Hash40,
    pub vanilla_expression_changed: bool,

    pub last_attackmodule_check_frame: i32,
    pub attackmodule_hitbox_detected_this_frame: bool,
    pub attackmodule_consecutive_detections: i32,
    pub last_hitbox_detection_method: HitboxDetectionMethod,

    pub evolution_start_pos_x: f32,
    pub evolution_start_pos_y: f32,

    pub is_evolving: bool,
    pub evolution_timer: i32,
    pub evolution_target_stage: EvolutionStage,
    pub linking_cord_active: bool,
    pub everstone_effect_active: bool,
    pub evo_attempt_delay_damage_taken_penalty: f32,
    pub evo_attempt_delay_hits_penalty: i32,

    pub current_frame: i32,
    pub last_debug_taunt_frame: i32,

    pub linking_cord_visual_icon_active: bool,
    pub linking_cord_visual_icon_timer: i32,
    pub up_taunt_press_count_linking_cord: u8,
    pub last_up_taunt_input_frame_linking_cord: i32,
    pub manual_linking_cord_evo_attempted_this_frame: bool,
    pub linking_cord_consumed_everstone_this_frame: bool,

    pub linking_cord_evo_attempt_icon_timer: i32,
    pub linking_cord_evo_attempt_icon_is_pos_sensitive: bool,

    pub everstone_icon_active: bool,
    pub everstone_icon_timer: i32,
    pub everstone_x_icon_active: bool,
    pub everstone_x_icon_timer: i32,
    pub special_press_count_everstone: u8,
    pub last_special_press_frame_everstone: i32,

    pub down_taunt_cancel_press_count: u8,
    pub last_down_taunt_cancel_input_frame: i32,

    pub mega_gengar_form_active: bool,
    pub giga_gengar_form_active: bool,
    pub is_in_final_smash_form: bool,
    pub gengarite_icon_display_active: bool,
    pub gengarite_icon_display_timer: i32,
    pub dynamax_icon_display_active: bool,
    pub dynamax_icon_display_timer: i32,

    pub gengarite_input_sequence: Vec<i32>,
    pub last_gengarite_input_frame: i32,

    pub s_taunt_r_press_count_dynamax: u8,
    pub last_s_taunt_r_input_frame_dynamax: i32,

    pub icon_start_pos_x: f32,
    pub icon_start_pos_y: f32,
    pub is_any_pos_sensitive_icon_active: bool,

    // --- Evolution Readiness Icons State ---
    pub dmg_t_icon_display_timer: i32,
    pub dmg_t_icon_is_locked_out: bool,

    pub dmg_d_icon_display_timer: i32,
    pub dmg_d_icon_is_locked_out: bool,

    pub dmg_ss_icon_display_timer: i32,
    pub dmg_ss_icon_is_locked_out: bool,

    pub dmg_se_icon_display_timer: i32,
    pub dmg_se_icon_is_locked_out: bool,

    pub last_evolution_confirmation_frame: i32,

    // Evolution completion tracking
    pub evolution_just_completed_this_frame: bool,
    pub frames_since_level_up_effect: i32,
    pub evolution_just_cancelled_this_frame: bool,
    pub evolution_cancel_fade_timer: i32,

    // REMOVED: Old aura tracking (now handled by gapless system)
    // pub current_aura_handle: Option<u32>,

    // Mewtwo shadowball effect tracking
    pub mewtwo_shadowball_hold_spawn_frame: i32,
    pub mewtwo_shadowball_spawned_this_rollout: bool,
    pub mewtwo_shadowball_hold_last_spawn_frame: i32,
    pub mewtwo_shadowball_hold_spawned_this_hold: bool,
    pub mewtwo_shadowball_growing_handle: Option<u32>,
    pub mewtwo_shadowball_growth_start_frame: i32,
    pub last_rollout_status: i32,
    pub last_hold_status: i32,

    // Shiny
    pub is_shiny: bool,
    pub shiny_effect_pending: bool,
    pub shiny_effect_delay_timer: i32,
    pub evolution_completion_frame: i32,
    pub delayed_cry_sound: String,
    pub delayed_cry_timer: i32,
}

impl PlayerEvolutionState {
    pub fn new() -> Self {
        Self {
            stage: EvolutionStage::Gastly,
            blink_timer: unsafe { random_module::rand_range_i32(120, 300) },
            blink_phase: BlinkPhase::Open,
            damage_received_this_stage: 0.0,
            hits_landed_this_stage: 0,
            previous_total_damage: 0.0,

            shadowball_status_frames: 0,
            is_in_shadowball_status: false,
            last_shadowball_status: -1,

            last_status_during_evolution: -1,

            shadowball_was_sufficiently_charged: false,
            shadowball_air_charge_count: 0,
            shadowball_previous_status: -1,

            opponent_damage_tracking: HashMap::new(),
            was_attacking_last_frame: false,

            vanilla_expression_tracking: true,
            last_vanilla_expression: Hash40 { hash: 0 },
            vanilla_expression_changed: false,

            last_attackmodule_check_frame: -1,
            attackmodule_hitbox_detected_this_frame: false,
            attackmodule_consecutive_detections: 0,
            last_hitbox_detection_method: HitboxDetectionMethod::None,

            evolution_start_pos_x: 0.0,
            evolution_start_pos_y: 0.0,

            is_evolving: false,
            evolution_timer: 0,
            evolution_target_stage: EvolutionStage::Gastly,
            linking_cord_active: false,
            everstone_effect_active: false,
            evo_attempt_delay_damage_taken_penalty: 0.0,
            evo_attempt_delay_hits_penalty: 0,
            current_frame: 0,
            last_debug_taunt_frame: -DEBUG_TAUNT_COOLDOWN -1,
            linking_cord_visual_icon_active: false,
            linking_cord_visual_icon_timer: 0,
            up_taunt_press_count_linking_cord: 0,
            last_up_taunt_input_frame_linking_cord: 0,
            manual_linking_cord_evo_attempted_this_frame: false,
            linking_cord_consumed_everstone_this_frame: false,
            linking_cord_evo_attempt_icon_timer: 0,
            linking_cord_evo_attempt_icon_is_pos_sensitive: false,
            everstone_icon_active: false,
            everstone_icon_timer: 0,
            everstone_x_icon_active: false,
            everstone_x_icon_timer: 0,
            special_press_count_everstone: 0,
            last_special_press_frame_everstone: 0,
            down_taunt_cancel_press_count: 0,
            last_down_taunt_cancel_input_frame: 0,
            mega_gengar_form_active: false,
            giga_gengar_form_active: false,
            is_in_final_smash_form: false,
            gengarite_icon_display_active: false,
            gengarite_icon_display_timer: 0,
            dynamax_icon_display_active: false,
            dynamax_icon_display_timer: 0,
            gengarite_input_sequence: Vec::with_capacity(MAX_TAUNT_SEQUENCE_LEN),
            last_gengarite_input_frame: 0,
            s_taunt_r_press_count_dynamax: 0,
            last_s_taunt_r_input_frame_dynamax: 0,
            icon_start_pos_x: 0.0,
            icon_start_pos_y: 0.0,
            is_any_pos_sensitive_icon_active: false,

            // Evolution Readiness Icons State
            dmg_t_icon_display_timer: 0,
            dmg_t_icon_is_locked_out: false,
            dmg_d_icon_display_timer: 0,
            dmg_d_icon_is_locked_out: false,
            dmg_ss_icon_display_timer: 0,
            dmg_ss_icon_is_locked_out: false,
            dmg_se_icon_display_timer: 0,
            dmg_se_icon_is_locked_out: false,
            last_evolution_confirmation_frame: -1,

            // Evolution completion tracking
            evolution_just_completed_this_frame: false,
            frames_since_level_up_effect: -1, // -1 means not active
            evolution_just_cancelled_this_frame: false,
            evolution_cancel_fade_timer: -1, // -1 means not active

            // REMOVED: Old aura tracking
            // current_aura_handle: None,

            // Mewtwo shadowball effect tracking
            mewtwo_shadowball_hold_spawn_frame: 0,
            mewtwo_shadowball_spawned_this_rollout: false,
            mewtwo_shadowball_hold_last_spawn_frame: 0,
            mewtwo_shadowball_hold_spawned_this_hold: false,
            mewtwo_shadowball_growing_handle: None,
            mewtwo_shadowball_growth_start_frame: 0,
            last_rollout_status: -1,
            last_hold_status: -1,

            // Shiny
            is_shiny: false,
            shiny_effect_pending: false,
            shiny_effect_delay_timer: -1,
            evolution_completion_frame: -1,
            delayed_cry_sound: String::new(),
            delayed_cry_timer: -1,
        }
    }

    // All other methods remain the same, just remove any references to current_aura_handle
    pub unsafe fn reset_for_new_stage(&mut self, new_stage: EvolutionStage, _my_boma: *mut BattleObjectModuleAccessor) {
        self.stage = new_stage;
        self.damage_received_this_stage = 0.0;
        self.hits_landed_this_stage = 0;

        // REMOVED: Old aura handle tracking
        // self.current_aura_handle = None;

        self.shadowball_status_frames = 0;
        self.is_in_shadowball_status = false;
        self.last_shadowball_status = -1;

        self.shadowball_was_sufficiently_charged = false;
        self.shadowball_air_charge_count = 0;
        self.shadowball_previous_status = -1;

        self.reset_vanilla_expression_state();
        self.reset_attackmodule_state();

        self.evolution_start_pos_x = 0.0;
        self.evolution_start_pos_y = 0.0;

        self.is_evolving = false;
        self.evolution_timer = 0;
        self.linking_cord_active = false;
        self.blink_phase = BlinkPhase::Open;
        self.blink_timer = unsafe { random_module::rand_range_i32(120, 300) };
        self.evo_attempt_delay_damage_taken_penalty = 0.0;
        self.evo_attempt_delay_hits_penalty = 0;

        self.linking_cord_visual_icon_active = false; self.linking_cord_visual_icon_timer = 0;
        self.up_taunt_press_count_linking_cord = 0; self.last_up_taunt_input_frame_linking_cord = 0;
        self.manual_linking_cord_evo_attempted_this_frame = false;
        self.linking_cord_consumed_everstone_this_frame = false;
        self.linking_cord_evo_attempt_icon_timer = 0;
        self.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
        self.everstone_effect_active = false;
        self.everstone_icon_active = false; self.everstone_icon_timer = 0;
        self.everstone_x_icon_active = false; self.everstone_x_icon_timer = 0;
        self.special_press_count_everstone = 0; self.last_special_press_frame_everstone = 0;
        self.down_taunt_cancel_press_count = 0; self.last_down_taunt_cancel_input_frame = 0;
        self.gengarite_icon_display_active = false; self.gengarite_icon_display_timer = 0;
        self.dynamax_icon_display_active = false; self.dynamax_icon_display_timer = 0;
        self.gengarite_input_sequence.clear(); self.last_gengarite_input_frame = 0;
        self.s_taunt_r_press_count_dynamax = 0; self.last_s_taunt_r_input_frame_dynamax = 0;
        self.is_in_final_smash_form = false;
        self.is_any_pos_sensitive_icon_active = false;

        // Reset completion flags
        self.evolution_just_completed_this_frame = false;
        self.frames_since_level_up_effect = -1;
        self.evolution_just_cancelled_this_frame = false;
        self.evolution_cancel_fade_timer = -1;

        // Reset Mewtwo effects
        self.mewtwo_shadowball_hold_spawn_frame = 0;
        self.mewtwo_shadowball_spawned_this_rollout = false;
        self.mewtwo_shadowball_hold_last_spawn_frame = 0;
        self.mewtwo_shadowball_hold_spawned_this_hold = false;
        self.mewtwo_shadowball_growing_handle = None;
        self.mewtwo_shadowball_growth_start_frame = 0;
        self.last_rollout_status = -1;
        self.last_hold_status = -1;

        // Shiny
        self.shiny_effect_pending = false;
        self.shiny_effect_delay_timer = -1;
        self.evolution_completion_frame = -1;
        self.delayed_cry_sound = String::new();
        self.delayed_cry_timer = -1;

        // Don't reset results screen stage - preserve across evolution

        self.reset_evo_readiness_icons();
    }

    pub unsafe fn full_reset_on_respawn(&mut self, boma: *mut BattleObjectModuleAccessor) {
        self.reset_for_new_stage(EvolutionStage::Gastly, boma);
        self.mega_gengar_form_active = false;
        self.giga_gengar_form_active = false;
        self.previous_total_damage = 0.0;

        // Ensure evolution progress is completely cleared
        self.damage_received_this_stage = 0.0;
        self.hits_landed_this_stage = 0;
        self.evo_attempt_delay_damage_taken_penalty = 0.0;
        self.evo_attempt_delay_hits_penalty = 0;
        
        self.last_evolution_confirmation_frame = -1;

        // Reset Mewtwo effects
        self.mewtwo_shadowball_spawned_this_rollout = false;
        self.mewtwo_shadowball_hold_last_spawn_frame = 0;
        self.mewtwo_shadowball_hold_spawned_this_hold = false;
        self.mewtwo_shadowball_growing_handle = None;
        self.mewtwo_shadowball_growth_start_frame = 0;
        self.last_rollout_status = -1;
        self.last_hold_status = -1;

        self.shiny_effect_pending = false;
        self.shiny_effect_delay_timer = -1;
        self.evolution_completion_frame = -1;

    }

    // Rest of the methods remain exactly the same...
    // (Including all the existing methods like reset_evo_readiness_icons, advance_blink_phase, etc.)
    
    pub fn reset_evo_readiness_icons(&mut self) {
        self.dmg_t_icon_display_timer = 0;
        self.dmg_t_icon_is_locked_out = false;
        self.dmg_d_icon_display_timer = 0;
        self.dmg_d_icon_is_locked_out = false;
        self.dmg_ss_icon_display_timer = 0;
        self.dmg_ss_icon_is_locked_out = false;
        self.dmg_se_icon_display_timer = 0;
        self.dmg_se_icon_is_locked_out = false;
    }

    pub fn advance_blink_phase(&mut self) {
        match self.blink_phase {
            BlinkPhase::Open => { self.blink_phase = BlinkPhase::HalfClose; self.blink_timer = 3; }
            BlinkPhase::HalfClose => { self.blink_phase = BlinkPhase::FullClose; self.blink_timer = 5; }
            BlinkPhase::FullClose => { self.blink_phase = BlinkPhase::HalfOpen; self.blink_timer = 3; }
            BlinkPhase::HalfOpen => { self.blink_phase = BlinkPhase::Open; self.blink_timer = unsafe { random_module::rand_range_i32(120, 300) }; }
        }
    }

    pub unsafe fn find_recently_hit_enemy(&self, my_boma: *mut BattleObjectModuleAccessor) -> Option<(u32, Vector3f)> {
        let my_entry_id = WorkModule::get_int(my_boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as u32;
        let my_pos = {
            let mut pos = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            ModelModule::joint_global_position(my_boma, Hash40::new("top"), &mut pos, true);
            pos
        };
        
        let mut closest_enemy: Option<(u32, Vector3f, f32)> = None; // (id, pos, distance)
        
        for enemy_id in 0..8u32 {
            if enemy_id == my_entry_id { continue; }
            
            let enemy_boma = smash::app::sv_battle_object::module_accessor(enemy_id);
            if enemy_boma.is_null() { continue; }
            
            // Check if enemy exists and has taken damage recently
            let enemy_kind = utility::get_kind(&mut *enemy_boma);
            if enemy_kind < *FIGHTER_KIND_MARIO || enemy_kind > *FIGHTER_KIND_PICKEL { continue; }
            
            // Get enemy hip position
            let mut enemy_hip_pos = Vector3f { x: 0.0, y: 0.0, z: 0.0 };
            ModelModule::joint_global_position(enemy_boma, Hash40::new("hip"), &mut enemy_hip_pos, true);
            
            // Calculate distance to find closest enemy
            let dx = enemy_hip_pos.x - my_pos.x;
            let dy = enemy_hip_pos.y - my_pos.y;
            let dz = enemy_hip_pos.z - my_pos.z;
            let distance = (dx*dx + dy*dy + dz*dz).sqrt();
            
            // Only consider enemies within reasonable range (10 units)
            if distance > 10.0 { continue; }
            
            match closest_enemy {
                None => closest_enemy = Some((enemy_id, enemy_hip_pos, distance)),
                Some((_, _, closest_dist)) => {
                    if distance < closest_dist {
                        closest_enemy = Some((enemy_id, enemy_hip_pos, distance));
                    }
                }
            }
        }
        
        closest_enemy.map(|(id, pos, _)| (id, pos))
    }
    
    // Helper to check if we're actively hitting someone with shadowball
    pub unsafe fn is_shadowball_hitting_enemy(&self, boma: *mut BattleObjectModuleAccessor) -> bool {
        let current_status = StatusModule::status_kind(boma);
        let is_rollout = current_status == PURIN_SPECIAL_N_ROLL || 
                         current_status == PURIN_SPECIAL_N_ROLL_AIR || 
                         current_status == PURIN_SPECIAL_N_TURN;
        
        if !is_rollout { return false; }
        
        // Multiple detection methods for reliability
        AttackModule::is_infliction_status(boma, 0) ||
        AttackModule::is_attack_occur(boma) ||
        AttackModule::is_attack(boma, 0, false) ||
        AttackModule::is_attack(boma, 1, false) ||
        AttackModule::is_attack(boma, 2, false)
    }

    pub fn reset_attackmodule_state(&mut self) {
        self.last_attackmodule_check_frame = -1;
        self.attackmodule_hitbox_detected_this_frame = false;
        self.attackmodule_consecutive_detections = 0;
        self.last_hitbox_detection_method = HitboxDetectionMethod::None;
    }

    unsafe fn update_attackmodule_detection_state(&mut self, boma: *mut BattleObjectModuleAccessor) {
        self.last_attackmodule_check_frame = self.current_frame;
        self.attackmodule_hitbox_detected_this_frame = false;

        if AttackModule::is_infliction_status(boma, 0) {
            self.attackmodule_hitbox_detected_this_frame = true;
            self.last_hitbox_detection_method = HitboxDetectionMethod::AttackModuleInfliction;
            self.attackmodule_consecutive_detections += 1;
            return;
        }

        if AttackModule::is_attack_occur(boma) {
            self.attackmodule_hitbox_detected_this_frame = true;
            self.last_hitbox_detection_method = HitboxDetectionMethod::AttackModuleOccur;
            self.attackmodule_consecutive_detections += 1;
            return;
        }

        for &attack_id in SHADOWBALL_HITBOX_IDS_TO_CHECK.iter() {
            if AttackModule::is_attack(boma, attack_id, false) {
                self.attackmodule_hitbox_detected_this_frame = true;
                self.last_hitbox_detection_method = HitboxDetectionMethod::AttackModuleSpecificId(attack_id);
                self.attackmodule_consecutive_detections += 1;
                return;
            }
        }

        if self.attackmodule_consecutive_detections > 0 {
            self.attackmodule_consecutive_detections = 0;
            self.last_hitbox_detection_method = HitboxDetectionMethod::None;
        }
    }

    pub unsafe fn detect_vanilla_expression_with_attackmodule(&mut self, boma: *mut BattleObjectModuleAccessor) -> Option<Hash40> {
        if !self.vanilla_expression_tracking || boma.is_null() {
            return None;
        }

        if self.current_frame % ATTACKMODULE_DETECTION_FREQUENCY == 0 {
            self.update_attackmodule_detection_state(boma);
        }

        if self.attackmodule_hitbox_detected_this_frame {
            let attack_expression = match self.stage {
                EvolutionStage::Gastly => *GASTLY_EYE_ATTACK,
                EvolutionStage::Haunter => *HAUNTER_EYE_ATTACK,
                EvolutionStage::Gengar => *GENGAR_EYE_ATTACK,
            };

            if ATTACKMODULE_DEBUG_LOGGING && self.current_frame % 30 == 0 {
            //    println!("[VEMD+AttackModule] Using attack expression due to hitbox detection: {:?}",
            //            self.last_hitbox_detection_method);
            }
            return Some(attack_expression);
        }
        None
    }

    pub unsafe fn detect_vanilla_expression(&mut self, boma: *mut BattleObjectModuleAccessor) -> Option<Hash40> {
        if !self.vanilla_expression_tracking {
            return None;
        }
        if boma.is_null() {
            return None;
        }

        if let Some(expression) = self.detect_vanilla_expression_with_attackmodule(boma) {
            return Some(expression);
        }

        if let Some(expression) = crate::gastly::animation_hooks::detect_expression_from_game_state(boma, self) {
            return Some(expression);
        }

        self.check_vanilla_mesh_visibility(boma)
    }

    unsafe fn check_vanilla_mesh_visibility(&mut self, boma: *mut BattleObjectModuleAccessor) -> Option<Hash40> {
        if self.current_frame % 2 != 0 {
            return None;
        }
        if boma.is_null() {
            return None;
        }

        let vanilla_expressions = [
            (*PURIN_VANILLA_EYE_N, "normal"),
            (*PURIN_VANILLA_EYE_BLINK, "blink"),
            (*PURIN_VANILLA_EYE_HALFBLINK1, "halfblink"),
            (*PURIN_VANILLA_EYE_ATTACK, "attack"),
            (*PURIN_VANILLA_EYE_CAPTURE, "capture"),
            (*PURIN_VANILLA_EYE_OUCH, "ouch"),
            (*PURIN_VANILLA_EYE_DOWN, "down"),
            (*PURIN_VANILLA_EYE_HEAVYATTACK, "heavyattack"),
        ];

        for (vanilla_mesh, expression_type) in vanilla_expressions.iter() {
            if vanilla_mesh.hash != 0 {
                let is_visible = VisibilityModule::is_visible_mesh(boma, vanilla_mesh.hash as c_uint);

                if is_visible {
                    let custom_expression = self.map_expression_type_to_custom(*expression_type);
                    if let Some(custom_expr) = custom_expression {
                        if self.last_vanilla_expression.hash != custom_expr.hash {
                            self.vanilla_expression_changed = true;
                            self.last_vanilla_expression = custom_expr;
                        }
                        return Some(custom_expr);
                    }
                }
            }
        }
        None
    }

    pub fn map_vanilla_to_custom_expression(&self, vanilla_eye: Hash40) -> Option<Hash40> {
        let vanilla_expression_type = if vanilla_eye.hash == PURIN_VANILLA_EYE_N.hash {
            "normal"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_BLINK.hash {
            "blink"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_HALFBLINK1.hash {
            "halfblink"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_ATTACK.hash {
            "attack"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_CAPTURE.hash {
            "capture"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_OUCH.hash {
            "ouch"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_DOWN.hash {
            "down"
        } else if vanilla_eye.hash == PURIN_VANILLA_EYE_HEAVYATTACK.hash {
            "heavyattack"
        } else {
            return None;
        };
        self.map_expression_type_to_custom(vanilla_expression_type)
    }

    fn map_expression_type_to_custom(&self, expression_type: &str) -> Option<Hash40> {
        match (expression_type, self.stage) {
            ("normal", EvolutionStage::Gastly) => Some(*GASTLY_EYE_N),
            ("blink", EvolutionStage::Gastly) => Some(*GASTLY_EYE_BLINK),
            ("halfblink", EvolutionStage::Gastly) => Some(*GASTLY_EYE_HALFBLINK1),
            ("attack", EvolutionStage::Gastly) => Some(*GASTLY_EYE_ATTACK),
            ("capture", EvolutionStage::Gastly) => Some(*GASTLY_EYE_CAPTURE),
            ("ouch", EvolutionStage::Gastly) => Some(*GASTLY_EYE_OUCH),
            ("down", EvolutionStage::Gastly) => Some(*GASTLY_EYE_DOWN),
            ("heavyattack", EvolutionStage::Gastly) => Some(*GASTLY_EYE_HEAVYATTACK),

            ("normal", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_N),
            ("blink", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_BLINK),
            ("halfblink", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_HALFBLINK1),
            ("attack", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_ATTACK),
            ("capture", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_CAPTURE),
            ("ouch", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_OUCH),
            ("down", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_DOWN),
            ("heavyattack", EvolutionStage::Haunter) => Some(*HAUNTER_EYE_HEAVYATTACK),

            ("normal", EvolutionStage::Gengar) => Some(*GENGAR_EYE_N),
            ("blink", EvolutionStage::Gengar) => Some(*GENGAR_EYE_BLINK),
            ("halfblink", EvolutionStage::Gengar) => Some(*GENGAR_EYE_HALFBLINK1),
            ("attack", EvolutionStage::Gengar) => Some(*GENGAR_EYE_ATTACK),
            ("capture", EvolutionStage::Gengar) => Some(*GENGAR_EYE_CAPTURE),
            ("ouch", EvolutionStage::Gengar) => Some(*GENGAR_EYE_OUCH),
            ("down", EvolutionStage::Gengar) => Some(*GENGAR_EYE_DOWN),
            ("heavyattack", EvolutionStage::Gengar) => Some(*GENGAR_EYE_HEAVYATTACK),
            _ => None,
        }
    }

    pub fn set_vanilla_expression_tracking(&mut self, enabled: bool) {
        self.vanilla_expression_tracking = enabled;
        if !enabled {
            self.last_vanilla_expression = Hash40 { hash: 0 };
            self.vanilla_expression_changed = false;
        }
    }

    pub fn reset_vanilla_expression_state(&mut self) {
        self.last_vanilla_expression = Hash40 { hash: 0 };
        self.vanilla_expression_changed = false;
    }

    pub fn start_evolution_process(&mut self, target_stage: EvolutionStage, fighter: &mut L2CFighterCommon, is_manual_trigger: bool) {
        let boma = fighter.module_accessor;
        if self.is_evolving {
            println!("[EVOLUTION] Already evolving - ignoring start request");
            return;
        }
        if self.stage == target_stage {
            println!("[EVOLUTION] Already at target stage - ignoring start request");
            return;
        }

        if !is_manual_trigger && self.everstone_effect_active {
            println!("[EVOLUTION] Everstone active - ignoring auto evolution");
            return;
        }

        if !is_manual_trigger {
            let (required_dmg_received, required_hits) = match self.stage {
                EvolutionStage::Gastly => (
                    GASTLY_EVO_DMG_RECEIVED_THRESHOLD + self.evo_attempt_delay_damage_taken_penalty,
                    GASTLY_EVO_HITS_THRESHOLD + self.evo_attempt_delay_hits_penalty
                ),
                EvolutionStage::Haunter => (
                    HAUNTER_EVO_DMG_RECEIVED_THRESHOLD + self.evo_attempt_delay_damage_taken_penalty,
                    HAUNTER_EVO_HITS_THRESHOLD + self.evo_attempt_delay_hits_penalty
                ),
                _ => {
                    println!("[EVOLUTION] Invalid stage for auto evolution");
                    return;
                }
            };

            let dmg_received_met = self.damage_received_this_stage >= required_dmg_received;
            let hits_met = self.hits_landed_this_stage >= required_hits;

            if !dmg_received_met || !hits_met {
                // println!("[EVOLUTION] Conditions not met - DMG: {}/{}, HITS: {}/{}", 
                //        self.damage_received_this_stage, required_dmg_received,
                //        self.hits_landed_this_stage, required_hits);
                return;
            }
        }

        // CRITICAL FIX: Set evolution state flags in correct order
        self.linking_cord_active = true;
        self.is_evolving = true;
        self.evolution_target_stage = target_stage;
        self.evolution_timer = 0;
        
        println!("[EVOLUTION] ★ STARTED EVOLUTION ★ - Frame: {}, Target: {:?}, Manual: {}", 
                self.current_frame, target_stage, is_manual_trigger);
        println!("[EVOLUTION] is_evolving = true, evolution_timer = 0");

        // Handle manual evolution icon
        if is_manual_trigger && self.stage == EvolutionStage::Haunter && target_stage == EvolutionStage::Gengar {
            unsafe { 
                crate::gastly::icon_management::enforce_icon_exclusivity(self, Some(*LINKING_CORD_ICON)); 
            }
            self.linking_cord_evo_attempt_icon_timer = MANUAL_EVO_HAUNTER_ICON_DURATION;
            self.linking_cord_evo_attempt_icon_is_pos_sensitive = true;
        }
        
        // CRITICAL FIX: Log the exact state that the sound system should detect
        println!("[EVOLUTION] Sound system should now detect: is_evolving={}, timer={}", 
                self.is_evolving, self.evolution_timer);
    }

    pub fn confirm_evolution(&mut self, fighter: &mut L2CFighterCommon) {
        let boma = fighter.module_accessor;

        // Kill aura when evolving away from Gastly
        if self.stage == EvolutionStage::Gastly {
            unsafe {
                crate::gastly::effects::kill_gastly_aura_on_evolution(boma);
            }
        }

        println!("[EVOLUTION] ★ CONFIRMING EVOLUTION ★ to {:?} at frame {}", 
                self.evolution_target_stage, self.current_frame);
        
        // CRITICAL FIX: Clear evolution flags BEFORE reset_for_new_stage
        let target_stage = self.evolution_target_stage;
        self.is_evolving = false;
        self.linking_cord_active = false;
        self.linking_cord_evo_attempt_icon_timer = 0;
        self.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
        
        println!("[EVOLUTION] Cleared evolution flags - is_evolving = false");

        unsafe { 
            self.reset_for_new_stage(target_stage, boma); 
        }
        self.previous_total_damage = unsafe { DamageModule::damage(boma, 0) };
        self.last_evolution_confirmation_frame = self.current_frame;

        unsafe {
            macros::COL_NORMAL(fighter);
            crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, self.stage);
            crate::gastly::visuals::set_active_eye_mesh(boma, self, None);
            crate::gastly::icon_management::update_is_any_pos_sensitive_icon_active_flag(self);
        }

        // Set the flag AFTER reset_for_new_stage so it doesn't get cleared
        self.evolution_just_completed_this_frame = true;
        
        println!("[EVOLUTION] ★ EVOLUTION CONFIRMED ★ - Now stage: {:?}", self.stage);
    }

    // UPDATED: Enhanced cancel_evolution with better sound cleanup
    pub fn cancel_evolution(&mut self, fighter: &mut L2CFighterCommon) {
        let boma = fighter.module_accessor;
        
        println!("[EVOLUTION] ★ CANCELLING EVOLUTION ★ at frame {}", self.current_frame);
        
        self.is_evolving = false;
        self.linking_cord_active = false;
        self.linking_cord_evo_attempt_icon_timer = 0;
        self.linking_cord_evo_attempt_icon_is_pos_sensitive = false;
        self.evolution_timer = 0;

        // NEW: Simple 15% damage penalty for cancellation
        self.evo_attempt_delay_damage_taken_penalty += 15.0;
        // No hits penalty - keep default hits requirement
        
        self.down_taunt_cancel_press_count = 0;
        self.last_evolution_confirmation_frame = -1;
        
        // Reset evolution progress for the current stage
        self.damage_received_this_stage = 0.0;
        self.hits_landed_this_stage = 0;
        
        // Reset all evolution readiness icons when cancelling evolution
        self.reset_evo_readiness_icons();
        
        // Set the flag so cancel effects play next frame
        self.evolution_just_cancelled_this_frame = true;

        unsafe {
            // Play cancel evolution sound
            use smash::app::lua_bind::SoundModule;
            use smash::phx::Hash40;
            SoundModule::play_se(boma, Hash40::new("cancel_evolve"), true, false, false, false, smash::app::enSEType(0));
            
            macros::COL_NORMAL(fighter);
            crate::gastly::visuals::update_body_and_unique_parts_visibility(boma, self.stage);
            crate::gastly::visuals::set_active_eye_mesh(boma, self, None);
            crate::gastly::icon_management::update_is_any_pos_sensitive_icon_active_flag(self);
            
            // Explicitly hide all readiness icon meshes
            ModelModule::set_mesh_visibility(boma, *STG1_DMG_T_ICON, false);
            ModelModule::set_mesh_visibility(boma, *STG1_DMG_D_ICON, false);
            ModelModule::set_mesh_visibility(boma, *STG2_DMG_SS_ICON, false);
            ModelModule::set_mesh_visibility(boma, *STG2_DMG_SE_ICON, false);
        }
        
        println!("[EVOLUTION] Cancel evolution complete - is_evolving = false");
    }

    pub fn get_attackmodule_status(&self) -> String {
        format!("AttackModule: method={:?}, consecutive={}, last_check_frame={}",
                self.last_hitbox_detection_method,
                self.attackmodule_consecutive_detections,
                self.last_attackmodule_check_frame)
    }

    pub fn is_attackmodule_detecting_hitbox(&self) -> bool {
        self.attackmodule_hitbox_detected_this_frame &&
        self.last_attackmodule_check_frame == self.current_frame
    }

    pub unsafe fn force_attackmodule_check(&mut self, boma: *mut BattleObjectModuleAccessor) {
        self.update_attackmodule_detection_state(boma);
    }

    pub unsafe fn log_detailed_attackmodule_state(&self, boma: *mut BattleObjectModuleAccessor, frame: i32) {
        if !ATTACKMODULE_DEBUG_LOGGING || frame % 180 != 0 {
            return;
        }

        let infliction = AttackModule::is_infliction_status(boma, 0);
        let occur = AttackModule::is_attack_occur(boma);
        let attack_0 = AttackModule::is_attack(boma, 0, false);
        let attack_1 = AttackModule::is_attack(boma, 1, false);
        let attack_2 = AttackModule::is_attack(boma, 2, false);

        if infliction || occur || attack_0 || attack_1 || attack_2 {
            println!("[ATTACKMODULE DETAILED Frame {}] infliction={}, occur={}, attacks=[{},{},{}], method={:?}, consecutive={}",
                    frame, infliction, occur, attack_0, attack_1, attack_2,
                    self.last_hitbox_detection_method, self.attackmodule_consecutive_detections);
        }
    }

    pub unsafe fn get_comprehensive_debug_report(&self, _boma: *mut BattleObjectModuleAccessor) -> String {
        let basic_state = format!(
            "Stage: {:?}, Frame: {}, Evolving: {}, InFS: {}",
            self.stage, self.current_frame, self.is_evolving, self.is_in_final_smash_form
        );

        let attackmodule_state = self.get_attackmodule_status();

        let expression_state = format!(
            "VEMD: {}, LastExpr: {:#x}, Changed: {}", 
            self.vanilla_expression_tracking,
            self.last_vanilla_expression.hash,
            self.vanilla_expression_changed
        );

        let shadowball_state = format!(
            "SB_Frames: {}, SB_Status: {}, InSB: {}",
            self.shadowball_status_frames,
            self.last_shadowball_status,
            self.is_in_shadowball_status
        );

        format!("{} | {} | {} | {}", basic_state, attackmodule_state, expression_state, shadowball_state)
    }

    pub fn get_attackmodule_performance_metrics(&self) -> (i32, bool, HitboxDetectionMethod) {
        (
            self.attackmodule_consecutive_detections,
            self.attackmodule_hitbox_detected_this_frame,
            self.last_hitbox_detection_method
        )
    }
}