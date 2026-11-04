use cantrip_api::*;
use cantrip_mechanics::{library, triggers};
use cantrip_oracle as oracle;

/// "Exile cards from the top of your library until you exile a nonland card. You may cast that card this turn." (CR 701.13a, 608.2c): only
/// the nonland card gets the permission, to cast it (never to play a land), paying its costs and following its timing (CR 601.2, 117.1a).
fn landfall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let (_, hit) = library::exile_until_nonland(g, item.controller, None, &|_, _| true);
    if let Some(card) = hit {
        g.allow_play(card, item.controller, PlayGrant { expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::LANDFALL,
        landfall,
        "Landfall — Whenever a land you control enters, exile cards from the top of your library until you exile a nonland card. You may cast that card this turn.",
    )],
    ..oracle::TERRITORIAL_BRUNTAR
};
