use cantrip_api::*;
use cantrip_mechanics::library::{search_to_hand, Find};
use cantrip_oracle as oracle;

/// Search your library for a land card, reveal it, put it into your hand, then shuffle.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| q.chars(c).is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{2}, {T}, Sacrifice Expedition Map: Search your library for a land card, reveal it, put it into your hand, then shuffle.",
        cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
        effect: search,
        ..ActivatedDef::BLANK
    })],
    ..oracle::EXPEDITION_MAP
};
