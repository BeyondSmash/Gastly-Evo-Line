// src/singletons.rs

use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use skyline::hooks::{getRegionAddress, Region};

// ****** ADD THESE USE STATEMENTS ******
use smash::app::BattleObjectModuleAccessor; // For function signatures if needed here
use smash::app::lua_bind::WorkModule;       // For WorkModule::get_int if used here
use smash::lib::lua_const::*;                     // For constants like FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID

// Assuming these app types are what your game version uses.
// If they are custom in your project, adjust the path.
pub use smash::app::BattleObject;
pub use smash::app::Fighter;
// pub use smash::app; // Alternative if you want to qualify with app::BattleObject

// Placeholder for cpp::Mutex if not readily available.
pub mod cpp {
    #[repr(C)]
    pub struct Mutex { _unused: [u8; 0x28] } // Example size
}

#[repr(C)]
pub struct BattleObjectTable { // Keep pub if other modules might need to inspect its structure directly
    vtable: *const *const (),
    objects: *mut BattleObject, // Changed to use the pub use'd BattleObject
    addresses: *mut u16,
    count: u32,
    mutex: cpp::Mutex,
}

#[repr(C)]
pub struct BattleObjectManagerInner { // Keep pub if BattleObjectManager derefs to it publicly
    active: *mut BattleObject,
    inactive: *mut BattleObject,
    unk: [u8; 0x90],
    pub fighters: BattleObjectTable, // Made pub for access from gastly/mod.rs if needed, or via methods
    weapons: BattleObjectTable,
    enemies: BattleObjectTable,
    gimmicks: BattleObjectTable,
    items: BattleObjectTable,
}

impl BattleObjectManagerInner {
    // This method is used by BattleObjectManager's Deref/DerefMut
    // Ensure Fighter type is correctly pathed
    pub fn fighters_mut(&mut self) -> CastedObjectIteratorMut<Fighter> {
        CastedObjectIteratorMut {
            start: self.fighters.objects,
            count: self.fighters.count as usize,
            current: 0,
            object_size: 0xf940, // This size might be character/version specific!
            _phantom: PhantomData,
        }
    }
    // Add other iterators like fighters() if needed
}

#[repr(C)]
pub struct BattleObjectManager { // ****** MAKE THIS PUBLIC ******
    inner: *mut BattleObjectManagerInner,
}

impl BattleObjectManager {
    // ****** MAKE THIS PUBLIC ******
    pub fn instance() -> Option<&'static Self> {
        unsafe {
            // IMPORTANT: The offset 0x52b5a00 must be correct for your game version!
            let addr = (getRegionAddress(Region::Text) as *const u8).add(0x52b5a00); // Ensure this offset is correct
            *(addr as *const Option<&'static Self>)
        }
    }

    // ****** MAKE THIS PUBLIC ******
    pub fn instance_mut() -> Option<&'static mut Self> {
        unsafe {
            // IMPORTANT: The offset 0x52b5a00 must be correct for your game version!
            let addr = (getRegionAddress(Region::Text) as *const u8).add(0x52b5a00); // Ensure this offset is correct
            std::ptr::read(addr as *const Option<&'static mut Self>)
        }
    }
}

impl Deref for BattleObjectManager {
    type Target = BattleObjectManagerInner;
    fn deref(&self) -> &Self::Target { unsafe { &*self.inner } }
}

impl DerefMut for BattleObjectManager {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.inner } }
}


// Iterators
#[repr(C)]
pub struct CastedObjectIterator<T: 'static> {
    start: *mut BattleObject, // Assuming tables point to BattleObject from which T is cast
    count: usize,
    current: usize,
    object_size: usize, // Size of the full T struct in the game's memory if iterating raw tables
    _phantom: PhantomData<T>,
}

impl<T: 'static> Iterator for CastedObjectIterator<T> {
    type Item = &'static T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            loop {
                if self.current >= self.count { return None; }
                // Assuming start points to an array of T, or BattleObject that can be cast to T
                let next_obj_ptr = (self.start as *mut u8).add(self.object_size * self.current) as *const BattleObject;
                self.current += 1;

                // This status check is from the example, assumes BattleObject has a status field.
                // Your actual BattleObject definition in smash::app might differ.
                // if (*next_obj_ptr).status < 3 { continue; }

                return Some(&*(next_obj_ptr as *const T));
            }
        }
    }
}

#[repr(C)]
pub struct CastedObjectIteratorMut<T: 'static> {
    start: *mut BattleObject,
    count: usize,
    current: usize,
    object_size: usize,
    _phantom: PhantomData<T>,
}

impl<T: 'static> Iterator for CastedObjectIteratorMut<T> {
    type Item = &'static mut T;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            loop {
                if self.current >= self.count { return None; }
                let next_obj_ptr = (self.start as *mut u8).add(self.object_size * self.current) as *mut BattleObject;
                self.current += 1;

                // This status check is from the example.
                // if (*next_obj_ptr).status < 3 { continue; }

                return Some(&mut *(next_obj_ptr as *mut T));
            }
        }
    }
}

// The BattleObjectIterator and BattleObjectIteratorMut from your full example would also go here
// if you intend to use `active_iter` etc. They rely on specific fields like `next` and `unknown_byte3`
// in the BattleObject struct. Ensure your `smash::app::BattleObject` has these if you use those iterators.