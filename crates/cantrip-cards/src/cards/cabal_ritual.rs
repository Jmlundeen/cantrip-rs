use cantrip_api::*;
use cantrip_oracle as oracle;

/// Add {B}{B}{B}; threshold (seven or more cards in your graveyard) adds {B}{B}{B}{B}{B} instead. The spell is on the
/// stack as it resolves, so it is not counted.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let amount = if cantrip_mechanics::threshold(g, player) { 5 } else { 3 };
    g.add_mana(player, Mana::B, amount);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::CABAL_RITUAL };
