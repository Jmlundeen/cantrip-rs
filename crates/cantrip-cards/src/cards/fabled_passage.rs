use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_oracle as oracle;

/// Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Then if you control
/// four or more lands, untap that land.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let arrived = search_to_battlefield(g, player, item.object, Find::ONE, true, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    let Some(&land) = arrived.first() else { return Ok(()) };
    let lands = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).count();
    if lands >= 4 {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Sacrifice Fabled Passage: Search your library for a basic land card, put it onto the battlefield tapped, then shuffle. Then if you control four or more lands, untap that land.",
        cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
        effect: search,
        ..ActivatedDef::BLANK
    })],
    ..oracle::FABLED_PASSAGE
};
