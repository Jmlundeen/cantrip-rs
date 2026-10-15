use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::effects;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "If this artifact would enter, you may discard a land card instead. If you do, put this artifact onto the battlefield. If
/// you don't, put it into its owner's graveyard." A replacement with a choice inside it (CR 614.12a), settled before the card
/// moves, wherever it is. A player with no land card in hand is never asked: the payment is impossible, and it goes to the
/// graveyard.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let player = entering.controller;
    let lands: Vec<ObjectId> = g.state().zone(Zone::hand(player)).iter().copied().filter(|&c| g.chars(c).is(CardType::Land)).collect();
    let owner = g.obj(entering.object).owner;
    if lands.is_empty() || !effects::ask_may(g, player, entering.object, Purpose::Optional)? {
        entering.instead = Some(Zone::graveyard(owner));
        return Ok(());
    }
    if let Some(land) = choose::card(g, player, entering.object, Purpose::Discard, lands)? {
        g.discard(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[mana::TAP_FOR_ANY_COLOR],
    ..oracle::MOX_DIAMOND
};
