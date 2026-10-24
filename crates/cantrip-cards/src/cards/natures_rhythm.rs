use cantrip_api::*;
use cantrip_mechanics::{harmonize, library, select};
use cantrip_oracle as oracle;

/// "Search your library for a creature card with mana value X or less, put it onto the battlefield, then shuffle."
/// X is the value chosen as the spell was cast, harmonize's {X} included (CR 107.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::creature_cards(select::up_to(1)).with(select::mana_value_at_most(item.x as u32));
    library::find_to_battlefield(g, item.controller, item.object, rule, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[harmonize::way(mana("{X}{G}{G}{G}{G}"), "Harmonize {X}{G}{G}{G}{G}")],
    ..oracle::NATURES_RHYTHM
};
