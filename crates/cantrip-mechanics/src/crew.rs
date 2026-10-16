//! Crew (CR 702.122), the Vehicles' keyword ability.

use cantrip_api::*;

/// "This Vehicle becomes an artifact creature until end of turn" (CR 301.7b): crew's effect, and Bladewheel Chariot's other
/// ability's, which taps two artifacts instead.
pub fn becomes_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(item.object, item.object, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::EndOfTurn);
    Ok(())
}

/// Crew N (CR 702.122a): "Tap any number of other untapped creatures you control with total power N or more: This
/// Vehicle becomes an artifact creature until end of turn." The cost is the core's (`Cost::tap_creatures`), which counts
/// a creature that crews as though its power were greater for more (CR 702.122b). Shorikai, Genesis Engine and Esika's
/// Chariot print it; `text` is the ability's full text, which a scenario's `Ability:` matches.
pub const fn crew(power: u8, text: &'static str) -> AbilityDef {
    ability(Cost { tap_creatures: Some(TapCreatures { power, crew: true }), ..Cost::NONE }, becomes_a_creature, text)
}
