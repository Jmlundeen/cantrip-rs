use cantrip_api::*;
use cantrip_mechanics::{choose, paradigm};
use cantrip_oracle as oracle;

/// "Exile cards from the top of your library until you exile cards with total mana value 4 or greater." Each card is
/// exiled one at a time and its mana value added (X is 0 in exile, CR 202.3e); a card a replacement effect sends
/// elsewhere was not exiled and adds nothing. It stops when the library has run out. "You may cast any number of
/// spells from among them without paying their mana costs": `choose::cast_any_number_free`, cast as this spell resolves
/// (CR 608.2g) at any timing; the lands among them are not spells.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut exiled: Vec<ObjectId> = Vec::new();
    let mut total = 0;
    while total < 4 {
        let Some(&top) = g.state().zone(Zone::library(you)).first() else { break };
        let card = g.exile(top);
        // No printed card replaces "would be exiled" today; a card sent elsewhere is not counted, as cascade does.
        if g.obj(card).zone.is(ZoneKind::Exile) {
            total += g.mana_value(card);
            exiled.push(card);
        }
    }
    choose::cast_any_number_free(g, you, item.object, &exiled)
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    paradigm: true,
    abilities: &[paradigm::TRIGGER],
    ..oracle::IMPROVISATION_CAPSTONE
};
