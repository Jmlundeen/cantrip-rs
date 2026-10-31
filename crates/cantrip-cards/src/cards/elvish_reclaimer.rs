use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::{library, select, targets};
use cantrip_oracle as oracle;

/// "This creature gets +2/+2 as long as there are three or more land cards in your graveyard" (CR 604.1, 611.3a) as the static
/// ability's `affects`: only the source itself, judged as the layers make it, asked again whenever characteristics are computed.
fn itself_with_three_land_cards_in_your_graveyard(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = q.state();
    source == affected
        && state.zone(Zone::graveyard(state.obj(source).controller)).iter().filter(|&&card| q.chars(card).is(CardType::Land)).count() >= 3
}

/// "Search your library for a land card, put it onto the battlefield tapped, then shuffle" (CR 701.23, 701.24).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::LAND_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_three_land_cards_in_your_graveyard,
        plus_pt::<2, 2>,
        "This creature gets +2/+2 as long as there are three or more land cards in your graveyard.",
    )],
    abilities: &[ability(
        cost("{2}, {T}").and(sacrifice(1, targets::filters::land)),
        search,
        "{2}, {T}, Sacrifice a land: Search your library for a land card, put it onto the battlefield tapped, then shuffle.",
    )],
    ..oracle::ELVISH_RECLAIMER
};
