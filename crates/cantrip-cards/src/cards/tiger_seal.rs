use cantrip_api::*;
use cantrip_mechanics::effects::untap_self;
use cantrip_mechanics::triggers::{YOUR_UPKEEP, YOU_DRAW_YOUR_SECOND_CARD};
use cantrip_oracle as oracle;

/// "Tap this creature" (CR 701.26a): the ability's source.
fn tap_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.tap(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_UPKEEP, tap_this, "At the beginning of your upkeep, tap this creature."),
        triggered(YOU_DRAW_YOUR_SECOND_CARD, untap_self, "Whenever you draw your second card each turn, untap this creature."),
    ],
    ..oracle::TIGER_SEAL
};
