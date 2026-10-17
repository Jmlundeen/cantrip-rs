use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Vibrant Outburst deals 3 damage to any target. Tap up to one target creature." (CR 120.3, 701.26a): each target is
/// judged on its own as the spell resolves (CR 608.2b), so one that became illegal is left alone and the other still
/// happens; the spell is countered only if every target is illegal.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    if let Some(creature) = targets::object(g, item, 1) {
        g.tap(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::ANY.labelled("Deals 3 damage to target"),
            targets::CREATURE.optional().labelled("Tap up to one target creature"),
        ],
        effect,
    ),
    ..oracle::VIBRANT_OUTBURST
};
