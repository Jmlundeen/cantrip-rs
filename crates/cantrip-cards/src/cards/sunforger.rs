use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::search_library;
use cantrip_oracle as oracle;

fn plus_four_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 4);
}

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
    let found = search_library(g, you, item.object, 1, true, quality(words.color(Color::Red), words.color(Color::White)))?;
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
        AbilityDef::Activated(ActivatedDef {
            text: "{R}{W}, Unattach this Equipment: Search your library for a red or white instant card with mana value 4 or less and cast that card without paying its mana cost. Then shuffle.",
            cost: Cost { mana: mana(0, 1, 0, 0, 1, 0), unattach_self: true, ..Cost::NONE },
            effect: search_and_cast,
            ..ActivatedDef::BLANK
        }),
        equip(mana(3, 0, 0, 0, 0, 0), "Equip {3}"),
    ],
    statics: &[StaticDef {
        text: "Equipped creature gets +4/+0.",
        layer: Layer::PtModify,
        affects: attached_object,
        apply: plus_four_power,
        ..StaticDef::BLANK
    }],
    ..oracle::SUNFORGER
};
