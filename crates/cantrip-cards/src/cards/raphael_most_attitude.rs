use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::top_of_library;
use cantrip_mechanics::triggers::{ANOTHER_CREATURE_YOU_CONTROL_ENTERS, THIS_ATTACKS};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Exile", "exile the top card of your library?", "", "Exile it", "Don't");

/// "You may exile the top card of your library": exiled with Raphael (CR 607.2a), where its attack trigger finds it.
fn exile_top_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let top = top_of_library(g, item.controller, 1);
    if !top.is_empty() && ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        g.exile_with(&top, g.linked(item), false);
    }
    Ok(())
}

/// "Until end of turn, you may play a card exiled with Raphael" (CR 607.2a, 611.2c): the permission changes no object's
/// characteristics or controller, so it also covers cards exiled with Raphael after this resolves, for the rest of the
/// turn (CR 611.2a). "A card" is one card, so the grant is a single play.
fn may_play_exiled_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.allow_play_exiled_with(g.linked(item), item.controller, PlayGrant { lands: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST }, true);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(ANOTHER_CREATURE_YOU_CONTROL_ENTERS, exile_top_card, "Alliance — Whenever another creature you control enters, you may exile the top card of your library."),
        triggered(THIS_ATTACKS, may_play_exiled_cards, "Whenever Raphael attacks, until end of turn, you may play a card exiled with Raphael."),
    ],
    ..oracle::RAPHAEL_MOST_ATTITUDE
};
