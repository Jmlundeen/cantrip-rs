use cantrip_api::*;
use cantrip_mechanics::{targets, library::{search_to_hand, Find}};
use cantrip_oracle as oracle;

/// Search your library for a card, put that card into your hand, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::DIABOLIC_INTENT
};
