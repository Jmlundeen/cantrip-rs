use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_mechanics::library::exile_top_play_until_end_of_next_turn;
use cantrip_oracle as oracle;

/// "{3}{R}, {T}: Exile the top card of your library. Until the end of your next turn, you may play that card." The
/// ability's controller plays it, even if they no longer control the land: a land card under the ordinary land-play
/// rules, a spell with all its costs and timing (CR 601.2, 305.1). An empty library exiles nothing.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_end_of_next_turn(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Plains) }, { sub(Subtype::Island) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(cost("{3}{R}, {T}"), exile_top, "{3}{R}, {T}: Exile the top card of your library. Until the end of your next turn, you may play that card."),
    ],
    ..oracle::CORI_MOUNTAIN_MONASTERY
};
