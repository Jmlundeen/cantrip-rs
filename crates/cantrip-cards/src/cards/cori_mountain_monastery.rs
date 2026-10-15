use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_mechanics::library::exile_top_may_play;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{3}{R}, {T}: Exile the top card of your library. Until the end of your next turn, you may play that card." The
/// ability's controller plays it, even if they no longer control the land: a land card under the ordinary land-play
/// rules, a spell with all its costs and timing (CR 601.2, 305.1). An empty library exiles nothing.
fn exile_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    exile_top_may_play(g, item.controller, 1, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Plains) }, { sub(Subtype::Island) }>),
    abilities: &[
        mana::tap_for(mana::add_r, Mana::R, "{T}: Add {R}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{R}, {T}: Exile the top card of your library. Until the end of your next turn, you may play that card.",
            cost: Cost { mana: mana(3, 0, 0, 0, 1, 0), tap: true, ..Cost::NONE },
            effect: exile_top,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CORI_MOUNTAIN_MONASTERY
};
