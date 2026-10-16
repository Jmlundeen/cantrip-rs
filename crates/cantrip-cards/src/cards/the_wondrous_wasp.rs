use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Tap up to one target creature. It loses all abilities for as long as The Wondrous Wasp remains on the battlefield"
/// (CR 611.2b, 613.1f): nothing at all if the Wasp has already gone, since the duration is over before it begins.
fn sting(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.tap(creature);
    g.add_effect(item.object, creature, EffectKind::LoseAllAbilities, Duration::WhileSourceOnBattlefield);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, sting, "Wasp's Sting \u{2014} When The Wondrous Wasp enters, tap up to one target creature. It loses all abilities for as long as The Wondrous Wasp remains on the battlefield.")
        .targets(&[targets::CREATURE.optional()])],
    ..oracle::THE_WONDROUS_WASP
};
