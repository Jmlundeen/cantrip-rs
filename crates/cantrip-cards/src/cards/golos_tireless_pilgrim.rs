use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// When Golos enters, you may search your library for a land card, put that card onto the battlefield tapped, then
/// shuffle.
fn fetch_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, true, |q, c| q.chars(c).is(CardType::Land))?;
    Ok(())
}

/// Exile the top three cards of your library, one at a time. You may play them this turn without paying their mana
/// costs: only a card that really arrived in exile is one of "them".
fn exile_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let grant = PlayGrant { instead: Some(ManaCost::ZERO), lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST };
    cantrip_mechanics::library::exile_top_may_play(g, player, 3, player, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, fetch_land, "When Golos enters, you may search your library for a land card, put that card onto the battlefield tapped, then shuffle."),
        ability(cost("{2}{W}{U}{B}{R}{G}"), exile_three, "{2}{W}{U}{B}{R}{G}: Exile the top three cards of your library. You may play them this turn without paying their mana costs."),
    ],
    ..oracle::GOLOS_TIRELESS_PILGRIM
};
