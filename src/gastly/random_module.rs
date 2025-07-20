// src/gastly/random_module.rs

use smash::app::sv_math;
use smash::hash40;
use skyline::libc::c_int;

// Helper function to get a random integer within a range.
// Used for blink timer randomization.
pub unsafe fn rand_range_i32(min: i32, max: i32) -> i32 {
    if min >= max { return min; } // Ensure min is less than max

    // Add 1 to max because sv_math::rand is exclusive for the upper bound with integer ranges.
    // sv_math::rand(hash, N) returns a value in [0, N-1].
    // So if we want a range [min, max] (inclusive), the size of the range is (max - min + 1).
    // We want to generate a random number from 0 to (max - min), then add min.
    let range_size = (max - min + 1) as c_int; 
    
    if range_size <= 0 { return min; } // Should not happen if min < max, but good for safety.

    // Using a unique hash for this specific random number generation
    // to avoid conflicts if other parts of the game use sv_math::rand with the same seed.
    let type_hash_u64 = hash40("fighter_gastly_blink_random_seed_v45"); // Keep your unique seed
    
    min + sv_math::rand(type_hash_u64, range_size) as i32
}