#![feature(
    concat_idents,
    proc_macro_hygiene
)]
#![allow(
    unused_imports,
    unused_macros,
    unused_variables,
    unused_assignments,
    unused_unsafe,
    non_upper_case_globals,
    non_snake_case,
    clippy::borrow_interior_mutable_const
)]

use hash40::hash40;
use smash::hash40;

pub static mut MARKED_COLORS: [bool; 256] = [false; 256];
pub static mut SHINY_COLORS: [bool; 256] = [false; 256];

/// Check if a fighter is a marked Gastly costume (only for Purin)
pub unsafe fn is_marked_gastly_costume(boma: *mut smash::app::BattleObjectModuleAccessor) -> bool {
    if boma.is_null() { return false; }
    
    // Only apply to Purin
    let fighter_kind = smash::app::utility::get_kind(&mut *boma);
    if fighter_kind != *smash::lib::lua_const::FIGHTER_KIND_PURIN {
        return false;
    }
    
    let color_id = smash::app::lua_bind::WorkModule::get_int(boma, *smash::lib::lua_const::FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    color_id < 256 && MARKED_COLORS[color_id]
}

/// Check if a fighter is a shiny Gastly costume (only for Purin)
pub unsafe fn is_shiny_gastly_costume(boma: *mut smash::app::BattleObjectModuleAccessor) -> bool {
    if boma.is_null() { return false; }
    
    // Only apply to Purin
    let fighter_kind = smash::app::utility::get_kind(&mut *boma);
    if fighter_kind != *smash::lib::lua_const::FIGHTER_KIND_PURIN {
        return false;
    }
    
    let color_id = smash::app::lua_bind::WorkModule::get_int(boma, *smash::lib::lua_const::FIGHTER_INSTANCE_WORK_ID_INT_COLOR) as usize;
    color_id < 256 && SHINY_COLORS[color_id]
}

mod gastly;
mod singletons;

// Function to check if all required dependencies are installed
pub fn check_deps() -> bool {
    let mut passed = true;

    for dep in [
        "rom:/skyline/plugins/libparam_config.nro",
        "rom:/skyline/plugins/libthe_csk_collection.nro",
        "rom:/skyline/plugins/libarcropolis.nro",
        "rom:/skyline/plugins/libnro_hook.nro",
        "rom:/skyline/plugins/libsmashline_plugin.nro",
    ] {
        if !std::path::Path::new(dep).is_file() {
            // Dependency missing - installation incomplete
            passed = false;
        }
    }

    passed
}

extern "C" fn mods_mounted(_ev: arcropolis_api::Event) {
    const FIGHTER_NAME: &str = "purin";
    const MARKER_FILE: &str = "gastly.marker";
    let mut lowest_color: i32 = -1;
    let mut marked_slots: Vec<i32> = vec![];
    
    // Regular Marker check loop
    for x in 0..256 {
        if let Ok(_) = std::fs::read(format!(
            "mods:/fighter/{}/model/body/c{:02}/{}",
            FIGHTER_NAME, x, MARKER_FILE
        )) {
            unsafe {
                marked_slots.push(x as _);
                MARKED_COLORS[x as usize] = true;
                if lowest_color == -1 {
                    lowest_color = x as _;
                }
            }
        }
    }

    // Shiny Marker check loop
    for x in 0..256 {
        if let Ok(_) = std::fs::read(format!(
            "mods:/fighter/{}/model/body/c{:02}/shiny.marker",
            FIGHTER_NAME, x
        )) {
            unsafe {
                SHINY_COLORS[x as usize] = true;
            }
            // Shiny slot detected
        }
    }


    if lowest_color == -1 {
        return;
    }

    let color_num = {
        unsafe {
            let mut index = lowest_color;
            while index < 256 && MARKED_COLORS[index as usize] {
                index += 1;
            }
            index - lowest_color
        }
    };

    // Add character database entries for different evolution stages
    
    // Evolving "Who's that Pokemon?" chara_4 UI entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_evolving"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("evolving"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    the_csk_collection_api::add_chara_layout_db_entry_info(
        the_csk_collection_api::CharacterLayoutDatabaseEntry {
            ui_layout_id: hash40("ui_chara_evolving_00"),
            clone_from_ui_layout_id: Some(hash40("ui_chara_purin_00")),
            ui_chara_id: the_csk_collection_api::Hash40Type::Overwrite(hash40("ui_chara_evolving")),
            ..Default::default()
        },
    );

    // Haunter chara_4 UI entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_haunter"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("haunter"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    the_csk_collection_api::add_chara_layout_db_entry_info(
        the_csk_collection_api::CharacterLayoutDatabaseEntry {
            ui_layout_id: hash40("ui_chara_haunter_00"),
            clone_from_ui_layout_id: Some(hash40("ui_chara_purin_00")),
            ui_chara_id: the_csk_collection_api::Hash40Type::Overwrite(hash40("ui_chara_haunter")),
            ..Default::default()
        },
    );

    // Gengar chara_4 UI entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_gengar"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("gengar"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    the_csk_collection_api::add_chara_layout_db_entry_info(
        the_csk_collection_api::CharacterLayoutDatabaseEntry {
            ui_layout_id: hash40("ui_chara_gengar_00"),
            clone_from_ui_layout_id: Some(hash40("ui_chara_purin_00")),
            ui_chara_id: the_csk_collection_api::Hash40Type::Overwrite(hash40("ui_chara_gengar")),
            ..Default::default()
        },
    );

    // Mega Gengar chara_6 UI entry (for cutin)
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_mega_gengar"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("mega_gengar"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    // ===== CHARA_6 UI ENTRIES FOR CUTINS =====

    // Haunter chara_6 (cutin) entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_haunter_00"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin_00")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("haunter_cutin"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    // Gengar chara_6 (cutin) entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_gengar_00"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin_00")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("gengar_cutin"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    // Mega Gengar chara_6 (cutin) entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_mega_gengar_00"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin_00")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("mega_gengar_cutin"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    // Giga Gengar chara_6 (cutin) entry
    the_csk_collection_api::add_chara_db_entry_info(
        the_csk_collection_api::CharacterDatabaseEntry {
            ui_chara_id: hash40("ui_chara_giga_gengar_00"),
            clone_from_ui_chara_id: Some(hash40("ui_chara_purin_00")),
            name_id: the_csk_collection_api::StringType::Overwrite(
                the_csk_collection_api::CStrCSK::new("giga_gengar_cutin"),
            ),
            disp_order: the_csk_collection_api::SignedByteType::Overwrite(-1),
            ..Default::default()
        },
    );

    // Allow all custom UIs for online play
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_evolving"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_haunter"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_gengar"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_mega_gengar"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_giga_gengar"));

    // Allow chara_6 cutins for online play
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_haunter_00"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_gengar_00"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_mega_gengar_00"));
    the_csk_collection_api::allow_ui_chara_hash_online(hash40("ui_chara_giga_gengar_00"));

    // Gastly mod initialization with detected colors
    
    // Install systems exactly as they were when hook comment-out worked
    crate::gastly::install();
    crate::gastly::install_frame_callbacks_with_costumes();
    crate::gastly::install_acmd_with_costumes();
    
}

#[skyline::main(name = "libgastly")]
pub fn main() {
    
    if !check_deps() {
        return;
    }
    

    unsafe {
        extern "C" {
            fn arcrop_register_event_callback(
                ty: arcropolis_api::Event,
                callback: arcropolis_api::EventCallbackFn,
            );
        }
        arcrop_register_event_callback(arcropolis_api::Event::ModFilesystemMounted, mods_mounted);
    }
}