# Gastly Evolution Mod - Complete Feature Overview

## 12,000+ lines of code 🔥

### Current Costumes (mod can be reslotted to any slot using gastly.marker file)

- Regular (c01)
- Shiny (c02)

## Core Evolution System

**Three-Stage Evolution Chain:**

- **Gastly** (starting form) → **Haunter** → **Gengar**
- Each stage has unique visuals, sounds, and behaviors
- Evolution requirements: Damage received + hits landed thresholds
- Real-time visual feedback showing evolution readiness

**Evolution Mechanics:**

- **Gastly → Haunter**: 35% damage received + 10 hits landed
- **Haunter → Gengar**: 50% damage received + 15 hits landed
- **Manual Evolution**: Haunter can evolve to Gengar via Up Taunt x2 while guarding (Linking Cord item)
- **Evolution Animation**: 240-frame (4 second window) transformation with special effects
- **Cancellation**: Down Taunt x2 (In Air or On Ground While Guarding) during evolving to cancel (with penalty). Penalty = current stage’s damage taken requirement + 15% damage needed to evolve. ie. If Gastly (1 cancel): (35% + 15% penalty); If Haunter (1 cancel): (50% + 15% penalty)

## Visual System

**Stage-Specific Meshes:**

Complete model replacements for each evolution stage-

- **Gastly**
- **Haunter**
- **Gengar**

**Eye Expression System:**

- 8 different eye expressions per stage (normal, blink, attack, capture, etc.)
- Automatic blinking with randomized, natural timing
- Expression changes based on status (attacks, damage, etc.)
- Vanilla Jigglypuff expression detection and translation for all 3 Pokemon stages

**Animation-Specific Meshes:**

- **Floor Shadow**: Special crouching animation meshes
- **Shadow Ball**: A stage-specific mesh copy of the corresponding Pokemon is seen posed & charging the shadow ball.
- **Ghost Form**: Down Taunt retro Pokemon ghost pixel art
- **Evolving Meshes**: Stage-specific during evolving window (uses model.nuanmb with a texture that matches similarly to the anime’s evolving visual - not just all white - if you look closely there’s some cyan/blue noise)
- **Tongue Mechanics**: Stage-specific long tongue visibility (grab pummel, down air)

## Audio System

**Stage-Specific Voices:**

- Complete voice replacement for all three stages
- **Attack Voices**: Random selection from 4-5 variants per stage
- **Taunt Voices**: Unique Up Taunt sounds
- **Cry Sounds**: Pokémon cries for each stage after evolving
- **Evolution Sounds**: Special evolution audio sequences

**Environmental Audio:**

- **Levitation Sounds**: Gastly/Haunter float with ethereal movement sounds + Gengar Run
- **Footsteps**: Only Gengar has footsteps (walking anims)

**Healing Sound Cues:**

- **Potion**:

Condition: Heal ≥15% damage from any starting damage, EXCEPT when G_RESTORE applies

- When it triggers: Using healing items that restore at least 15% health
- **Full Restore**:

Condition: Heal from ≥35% damage to 0% damage

- When it triggers: Using healing items (like heart containers) that completely restore health when you had at least 35% damage
- **Pokecenter Full Heal Jingle**: You’ll hear the sfx during respawn/rebirth if you had 100% damage or greater upon the prior death

**Misc. Audio/Move Concept Change:**

- Replaced Star KO, Knockout (stamina)
- Stun by Mewtwo and deku nut item = Pokemon “Confused” Status SFX
- Up Special = Hypnosis (replaces Jigglypuff’s Sing)
- Side Special = Sucker Punch

## Item/Icon System

**Evolution Items:**

- **Everstone**: Toggle to prevent auto-evolution (Special x2 while guarding)
- **Everstone X**: Visual indicator when Everstone is disabled (Special x2 while guarding | also occurs when trying to use Everstone while evolving [it’s too late for that])
- **Linking Cord**: Manual evolution trigger for Haunter
- **Position-Sensitive Icons**: Items appear above character and disappear if player moves

**Final Smash Items (Gengar only):**

- **Gengarite Item**: Activates Mega Gengar form (Side Taunt Left x2)
- **Dynamax Symbol**: Activates Gigantamax Gengar form (Side Taunt Right x2)

**Evolution Readiness/Conditional Indicators:**

- **Damage Taken  / Hit-Count Icons**: Show when individual conditions are met
- **Evolve SS (Success) / Evolve SE (Start Evolving - Plays Level Up SFX) Icons**: Show when both conditions are met
- **UI Flash**: Eye flash when all icon meshes occur

## Final Smash System

**Multiple Forms for Gengar:**

- **Default**: Regular Gengar
- **Mega Gengar**: Different model, ending de-transform effect, and sounds
- **Gigantamax Gengar**: Different model, ending de-transform effect, and sounds
- **Form Selection**: Pre-select form before using Final Smash (Mega = Gengarite | Gigantamax = Dynamax)

**Visual Management:**

- **Aura Suppression**: Removes default Final Smash aura effects
- **Model Swapping**: Transitioning between forms (Gastly, Haunter, Gengar, Mega Gengar, Gigantamax Gengar)

## Dark Effects System

**Shadow-Type Attacks:**

- **Dark Moves**: Down tilt, down smash, neutral air, forward air, down special
- **Visual Effects**: Purple darkness effects and purple tinting on hit opponents
- **Status Effects**: Different effects based on move type

**Effect Types:**

- **Purple Darkness**: Standard darkness effect
- **Poison Effect**: Down special creates lingering poison
- Licking splatter: Down air (Gastly and Haunter only) and pummel spawn licking effects

## Shiny Pokémon System

**Shiny Detection:**

- **Marker System**: shiny.marker file detect shiny costumes
- **Visual Effects**: Sparkle effects
- **Timing**: Appears on entry and after evolution completion
- **Audio Cue:**  Sparkle occurs before the cry

## Weakened State System

**High Damage Effects:**

- **Threshold**: Activates at 150%+ damage
- **Visual**: Tired/weakened eye expressions
- **Effect**: Sweat dripping for fatigue
- **Exclusions**: Disabled during certain animations and statuses

## Neutral Special Replacement

**Shadow Ball System:**

- **Visual States**: Different meshes/effects for charging, charge_max, and release
- **Invisibility**: Character model becomes invisible during charged rollout and substituted by shadow ball effects
- **Release Audio**: Different sounds for charge_max release vs regular release
- **Bomb Effects**: Mewtwo-style explosion effects on hit

## UI Integration

**Portrait System:**

- **Dynamic Portraits**: Battle portraits change based on evolution stage
- **Cutin Effects**: Special portrait cutins during Pokemon cry (currently disabled due to not being a working feature for The CSK Collection)
- **Stock Icons**: Shadow symbol stock icons (with a sparkle if on shiny slot). May replace this with Pokemon stage-specific stock icons since it’s confirmed to work (including during mega and Gigantamax final smash) in the future
- **“Who's That Pokémon”**: Special "?" portrait during evolution

**Menu Integration:**

- **CSK Collection API**: Full integration with character selection mods
- **Online Compatibility**: All UI elements work in online play

## Technical Features

**Costume Filtering:**

- **Marker System**: (gastly.marker | shiny.marker*) Only specific costume slots have the mod active
- **Selective Installation**: ACMD and hooks only apply to marked costumes
- **Compatibility**: Non-marked costumes remain vanilla Jigglypuff
* = shiny sparkle effect/sound plays for shiny.marker slots

**Training Mode Support:**

- **Reset Detection**: Automatically resets evolution progress
- **State Persistence**: Maintains appropriate evolution state
- **Debug Features**: Extensive logging for troubleshooting

## **Effect Systems**

### **Gastly Gas Aura**

- **Persistent Gas Cloud**: Floating aura effect (RGB: 0.6, 0.35, 0.7) with position/rotation offsets
- **Evolution-Responsive RGB**: Aura changes to bright white (RGB: 7.0, 7.0, 7.0) during Gastly→Haunter evolution
- **Intelligent Suppression**: Automatically hides during swimming, rollout hitboxes, floor shadow crouching, specific motions
- **Dual-Bone System**: Switches between `body` bone and `shadowball` bone based on Shadow Ball state
- **Shiny Variant Support**: Blue aura (RGB: 0.42, 0.75, 1.3) for shiny costume slots

### **Shadow Ball**

- **Invisibility During Rollout**: Character becomes invisible during charged rollout with active hitboxes
- **Mewtwo Effect Replacement**: Complete Shadow Ball effect overhaul with charging, and projectile states
- **Speed Booster Effect**: Purple speed lines during invisible charged rollout
- **Frame Detection**: Multiple detection methods for charge states and hitbox activity - when hitbox is active (release) = shadow ball effects will be visible
- **Bomb Effects**: Custom explosion effect on rollout collision

### **Final Smash**

- **Ending Purple De-transform Effect**: During Mega/Giga Gengar forms
- **Mega Symbol**: Spawns when player has Final Smash capability + Mega mode are active
- **Distortion Screech**: Effect during climax of final smash

### **Advanced Animation Integration**

- **ACMD Effect Overrides**: Custom effects for 25+ attack animations with bone-specific targeting
- **Grab System Effects**: Special aura effects on hand bones during all grab types
- **Dark-Type Attack Effects**: Purple darkness effects on specific attacks (down tilt, down smash, aerials)
- **Taunt Effect System**: Custom effects for down taunt at frames 1 and 90

### **- 43 Wi-Fi Safe Spliced Animations! -**

### Victory Themes

- Each Win animation has its own special background stage
- Win1  = “Forest Manor” / “Old Chateau” - Pokémon Platinum
- Win2 = “Lavender Town” - Pokémon Red / Blue
- Win3 = “Spotted! (Pokémon Collector Version)” - Pokémon Platinum

### Credits

- BeyondSmash
- LilyLambda (Wi-Fi Safety Testing)
- CSK (The CSK Collection)
- Donsveertje (Thumbnail background base):
- [https://www.deviantart.com/donsveertje/art/Gastly-Haunter-and-Gengar-wallpaper-pack-858454269
-](https://www.deviantart.com/donsveertje/art/Gastly-Haunter-and-Gengar-wallpaper-pack-858454269) [https://x.com/Donsveertje/status/1317093350646960128](https://x.com/Donsveertje/status/1317093350646960128)
- GameFreak / Nintendo / Pokemon Company

### Current limitations during development:

- **Incompatible with One-Slot Effects plugin** - causes a soft-lock/freeze on any character (global) if they throw an enemy. I managed to comment out an install function in code and it didn't freeze on my laptop (although the function is necessary), yet doing the same method froze on my desktop, so I decided to not utilize it.
- Dark effects purple overlay is based on camera depth due to how Flash macro is configured so it's not a fixed color shade overlay. It gets more opaque as the camera is zoomed out. `ColorBlendModule::set_main_color` caused the model to be completely gray at all times and configurations.
- I wanted a "damage % dealt to enemy" condition, but too complex to incorporate, so it was replaced with hit-count requirement to evolve in addition to the damage taken condition.
- Ghost (down taunt) not billboarded due to facing left causing issues visually (perhaps flip.prc related)
- Long tongue mesh/bones not flipped/mirrored properly when facing left during grab/pummel (perhaps flip.prc related)
- Can't get Gastly aura to occur during results screen lose status potentially due to results screen UI render layer issue
- Regarding dark/lick hits: Common vanilla sounds that are hit sounds -- I wanted to mute/stop them during my custom hit sounds, but it requires global fighter frame to mute/stop them seemingly, and I didn't care to add that complexity in case it interfered with other occurrences of that common sound playing. It's not a big deal anyway.
- Custom `chara_6` cutin UI was supposed to occur when Haunter and Gengar did their cry after evolving and their final smash. Also was for Mega Gengar final smash and Gigantamax Gengar final smash. Unfortunately, the cutin only shows up as black (The CSK Collection limitation).
- Custom `chara_3` swapping was a consideration for the results screen evolution stage outcome, but was not utilized due to chara_6 not working, so I considered chara_3 to not work or not worth testing it.
- The evolving sound(s) may cease if you are interrupted by a grab from an enemy.
- In Gastly duos (Gastly vs. Gastly), when a shiny Gastly (Player 2) dies as Haunter/Gengar and respawns as Gastly, two things happen:
1. Regular Gastly's UI flickers between Gastly ↔ Haunter/Gengar stages - it may, after some seconds, correctly become the correct evolution stage UI
2. Shiny Gastly's UI gets stuck showing Haunter/Gengar instead of updating to Gastly after respawn
    
    Suspected Root Cause: Despite implementing instance isolation with unique instance_key calculations and per-player UI state management, the the_csk_collection_api::change_entry_chara_ui() API
    appears to have some global behavior or shared state that causes cross-player interference. Multiple debugging approaches (stage change detection, staggered updates, validation layers, complete, UI resets) all failed to resolve the issue.
    

---

### Things I could add, but felt lazy/not as necessary/didn't want the project to drag on longer **(may be added as an update in the future)**:

- Moving eyes (not sure to what extent Jigglypuff's internal eye movement's noticeability or will translate to the kind of eyes Gastly evo has)
- Final smash eyes (would have to make sure the EyeL/EyeGL# structure works for this mod)
- Red bubbly clouds above Gigantamax Gengar (would have to model, create new bones animate them, code in new mesh - all for like 3-4 seconds of visibility. You could argue the evolution sequence was similarly short, but the evolution sequence was the most critical piece for this mod project)
- Custom chara_4 / chara_2 that occurs during Mega Gengar and Gigantamax Gengar
- Custom chara_2 stock icons for each stage (for now it's just the Pokemon shadow symbol / Pokemon shadow symbol + sparkle)
- More recolor slots (current ideas: 1) [https://x.com/TheYisusOne/status/1944396469601001590?t=SRxpQLMtqr8_OuIzuqOB6w&s=19](https://x.com/TheYisusOne/status/1944396469601001590?t=SRxpQLMtqr8_OuIzuqOB6w&s=19), 2) cel-shaded anime colors, 3) other ghost Pokemon color palette references)