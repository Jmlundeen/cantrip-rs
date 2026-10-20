use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Spells and abilities your opponents control can't cause their controller to search their library" (CR 101.2, 701.23):
/// refused when an opponent of Ashiok's controller controls the spell or ability and so searches (`player` is the
/// causer's own controller) their own library. Spells and abilities that make someone else search, or search another
/// player's library, are not named.
fn opponents_cant_search(_: &dyn Query, asker: &Asker, causer: PlayerId, player: PlayerId, kind: CauseKind) -> bool {
    !(matches!(kind, CauseKind::Search { library } if library == player) && player == causer && causer != asker.controller)
}

/// "-1: Target player mills four cards. Then exile each opponent's graveyard." (CR 701.17, 701.13): the cards milled are
/// in their owner's graveyard when it is exiled, so an opponent's own milled cards go too (the 2019-05-03 ruling).
fn minus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.mill(player, 4);
    }
    effects::exile_each_opponents_graveyard(g, item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cause_rules: &[opponents_cant_search],
    abilities: &[ability(loyalty(-1), minus_one, "\u{2212}1: Target player mills four cards. Then exile each opponent's graveyard.")
        .targets(&[targets::PLAYER])],
    ..oracle::ASHIOK_DREAM_RENDER
};
