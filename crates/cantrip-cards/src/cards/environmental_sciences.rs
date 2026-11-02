use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, reveal it, put it into your hand, then shuffle. You gain 2 life." (CR 701.23, 119.3).
fn search_then_gain_2_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    lands::search_basic_land_to_hand(g, item.controller, item.object)?;
    g.end_sentence();
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search_then_gain_2_life),
    ..oracle::ENVIRONMENTAL_SCIENCES
};
