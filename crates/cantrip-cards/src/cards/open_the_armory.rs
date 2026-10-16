use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_oracle as oracle;

/// Search your library for an Aura or Equipment card, reveal it, put it into your hand, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    search_to_hand(g, player, item.object, Find::ONE.revealed(), |q, c| {
        let chars = q.chars(c);
        chars.has_subtype(Subtype::Aura) || chars.has_subtype(Subtype::Equipment)
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::OPEN_THE_ARMORY };
