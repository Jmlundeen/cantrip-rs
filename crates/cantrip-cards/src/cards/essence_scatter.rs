use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Counter target creature spell" (CR 701.6): a spell with the card type creature, an artifact creature included.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_SPELL], effect),
    ..oracle::ESSENCE_SCATTER
};
