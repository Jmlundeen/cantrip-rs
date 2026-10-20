use cantrip_api::*;
use cantrip_mechanics::{history::an_opponent_lost_life, library, triggers};
use cantrip_oracle as oracle;

/// "Exile the top two cards of your library. Choose one of them. Until end of turn, you may play that card." Both cards
/// are exiled together; only the one chosen can be played this turn, a land under the ordinary land-play rules (CR 305.1).
fn exile_two_and_choose_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let grant = PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST };
    library::exile_top_choose_one_may_play(g, item.controller, item.object, 2, grant)?;
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
