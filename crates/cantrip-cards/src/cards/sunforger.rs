use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::search_library;
use cantrip_oracle as oracle;

/// A red or white instant card with mana value 4 or less.
fn quality(red: Color, white: Color) -> impl Fn(&dyn Query, ObjectId) -> bool {
    move |q, card| {
        let c = q.chars(card);
        c.is(CardType::Instant) && (c.colors.has(red) || c.colors.has(white)) && q.mana_value(card) <= 4
    }
}

/// "Search your library for a red or white instant card with mana value 4 or less and cast that card without paying
/// its mana cost. Then shuffle." A search with a stated quality may find nothing (CR 701.23b); the card found is cast
/// from the library as the ability resolves (CR 608.2g), for no mana (CR 118.9), before the shuffle. A card that cannot
/// be cast stays in the library and is shuffled with it.
fn search_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let words = g.words(item.object);
    let found = search_library(g, you, you, item.object, SearchDestination::CastFree, 1, true, quality(words.color(Color::Red), words.color(Color::White)))?;
    if let Some(&card) = found.first()
        && g.could_cast_during_resolution(card)
    {
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
    g.shuffle(you);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{W}{R}").and(UNATTACH_THIS), search_and_cast, "{R}{W}, Unattach this Equipment: Search your library for a red or white instant card with mana value 4 or less and cast that card without paying its mana cost. Then shuffle."),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<4, 0>, "Equipped creature gets +4/+0.")],
    ..oracle::SUNFORGER
};
