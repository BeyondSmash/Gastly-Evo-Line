# Gastly Evolution Mod - Complete Feature Overview

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
- **Shadow Ball**: A stage-specific mesh copy of the corresponding pokemon is seen posed & charging the shadow ball.
- **Ghost Form**: Down Taunt retro Pokemon ghost pixel art
- **Evolving Meshes**: Stage-specific during evolving window (uses model.nuanmb with a texture that matches similarly to the anime’s evolving visual - not just all white - if you look closely there’s some cyan/blue noise)
- **Tongue Mechanics**: Stage-specific long tongue visibility (grab pummel, down air)

## Audio System

**Stage-Specific Voices:**

- Complete voice replacement for all three stages
- **Attack Voices**: Random selection from 4-5 variants per stage
- **Appeal Voices**: Unique Up Taunt sounds
- **Cry Sounds**: Pokémon cries for each stage after evolving
- **Evolution Sounds**: Special evolution audio sequences

**Environmental Audio:**

- **Levitation Sounds**: Gastly/Haunter float with ethereal movement sounds + Gengar Run
- **Footsteps**: Only Gengar has footsteps (walk anims)

## Item/Tool System

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
- **Evolve SS (Success) / Evolve SE (Start Evolving) Icons**: Show when both conditions are met
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

## Special Attack Enhancements

**Shadow Ball System:**

- **Visual States**: Different meshes/effects for charging, charge_max, and release
- **Invisibility**: Character model becomes invisible during charged rollout and substituted by shadow ball effects
- **Release Audio**: Different sounds for charge_max release vs regular release
- **Bomb Effects**: Mewtwo-style explosion effects on hit

## UI Integration

**Portrait System:**

- **Dynamic Portraits**: Battle portraits change based on evolution stage
- **Cutin Effects**: Special portrait cutins during pokemon cry (currently disabled due to not being a working feature for The CSK Collection)
- **Stock Icons**: Shadow symbol stock icons (with a sparkle if on shiny slot). May replace this with pokemon stage-specific stock icons since it’s confirmed to work (including during mega and gigantamax final smash) in the future
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
- **Intelligent Suppression**: Automatically hides during swimming, rollout hitboxes, specific motions
- **Dual-Bone System**: Switches between `body` bone and `shadowball` bone based on Shadow Ball state
- **Shiny Variant Support**: Blue aura (RGB: 0.42, 0.75, 1.3) for shiny costume slots

### **Shadow Ball**

- **Invisibility During Rollout**: Character becomes invisible during charged rollout with active hitboxes
- **Mewtwo Effect Replacement**: Complete Shadow Ball effect overhaul with charging, holding, and projectile states
- **Speed Booster Effect**: Purple speed lines during invisible charged rollout
- **Frame-Perfect Detection**: Multiple detection methods for charge states and hitbox activity - when hitbox is active (release) = shadow ball effects will be visible
- **Bomb Effects**: Custom explosion effect on rollout collision

### **Final Smash**

- **Ending Purple De-transform Effect**: During Mega/Giga Gengar forms
- **Mega Symbol**: Spawns when both player has Final Smash capability + Mega mode are active
- **Distortion Screech**: Effect during climax of final smash

### **Advanced Animation Integration**

- **ACMD Effect Overrides**: Custom effects for 25+ attack animations with bone-specific targeting
- **Grab System Effects**: Special aura effects on hand bones during all grab types
- **Dark-Type Attack Effects**: Purple darkness effects on specific attacks (down tilt, down smash, aerials)
- **Taunt Effect System**: Custom effects for down taunt at frames 1 and 90