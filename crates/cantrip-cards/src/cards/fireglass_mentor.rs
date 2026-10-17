use cantrip_api::*;
use cantrip_mechanics::{choose, history::an_opponent_lost_life, top_of_library, triggers};
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Choose one of them. Until end of turn, you may play that card." Both cards
/// are exiled together; the one chosen (from those that really reached exile) can be played this turn, a land under the
/// ordinary land-play rules (CR 305.1), and the other can't be.
fn exile_two_and_choose_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let top = top_of_library(g, item.controller, 2);
    let exiled: Vec<ObjectId> = g.exile_all(&top).into_iter().filter(|&c| g.obj(c).zone.is(ZoneKind::Exile)).collect();
    if let Some(chosen) = choose::card(g, item.controller, item.object, Purpose::Play, exiled)? {
        g.allow_play(chosen, item.controller, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_SECOND_MAIN_PHASE,
        exile_two_and_choose_one,
        "At the beginning of your second main phase, if an opponent lost life this turn, exile the top two cards of your library. Choose one of them. Until end of turn, you may play that card.",
    )
    .condition(an_opponent_lost_life)],
    ..oracle::FIREGLASS_MENTOR
};
