use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Exile up to one target card from a graveyard. Target creature you control connives." (CR 701.13, 701.50, 608.2b): two targets
/// chosen as the ability goes on the stack, the card optional. Each is dealt with only if it is still legal as the ability
/// resolves; one gone does not stop the other.
fn exile_then_connive(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    let Some(creature) = targets::object(g, item, 1) else { return Ok(()) };
    cantrip_mechanics::connive(g, creature, item.object)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        exile_then_connive,
        "When this creature enters, exile up to one target card from a graveyard. Target creature you control connives. (Draw a card, then discard a card. If you discarded a nonland card, put a +1/+1 counter on that creature.)",
    )
    .targets(&[targets::CARD_IN_A_GRAVEYARD.optional(), targets::CREATURE_YOU_CONTROL])],
    ..oracle::MECHANICAL_MOBSTER
};
