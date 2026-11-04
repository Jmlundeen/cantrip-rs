//! Triple (CR 701.11): to triple a creature's power and/or toughness is to give it +X/+0 and/or +0/+Y for twice the power X
//! and twice the toughness Y it has as the effect resolves (CR 701.11b); a negative value gives as much less (CR 701.11c).
//! Tifa's Limit Break triples power and toughness.

use cantrip_api::*;

/// Triple `creature`'s power and toughness until end of turn (CR 701.11a-c): +2X/+2Y for the power X and toughness Y it has
/// now, each read once and fixed (CR 608.2h); a negative value gives as much less (-X/+Y, +X/-Y or -X/-Y). Tifa's Limit Break
/// prints it.
pub fn power_and_toughness(g: &mut dyn Game, source: ObjectId, creature: ObjectId) {
    let chars = g.chars(creature);
    let (power, toughness) = (chars.power.unwrap_or(0), chars.toughness.unwrap_or(0));
    g.add_effect(source, creature, EffectKind::Pump(2 * power, 2 * toughness), Duration::EndOfTurn);
}
