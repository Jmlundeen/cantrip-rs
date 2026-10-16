use cantrip_api::*;
use cantrip_mechanics::{triggers, effects};
use cantrip_oracle as oracle;

/// That player exiles cards from the top of their library until they exile a nonland card. You may put a +1/+1
/// counter on Black Widow. If you don't, you may cast the exiled nonland card until end of turn and mana of any type
/// can be spent to cast that spell (CR 118.14).
fn steal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(victim) = item.subject else { return Ok(()) };
    let mut nonland = None;
    while let Some(&top) = g.state().zone(Zone::library(victim)).first() {
        let is_land = g.chars(top).is(CardType::Land);
        let exiled = g.exile(top);
        if !is_land {
            nonland = Some(exiled);
            break;
        }
    }
    // CR 608.2d: no choice is offered that cannot be carried out, so Black Widow gone from the battlefield is not asked
    // for a counter; the counter was not put on her, and "if you don't" holds.
    let on_battlefield = is_live(g, item.object) && g.obj(item.object).zone.is(ZoneKind::Battlefield);
    if on_battlefield && effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    } else if let Some(card) = nonland
        && g.obj(card).zone.is(ZoneKind::Exile)
    {
        g.allow_play(card, item.controller, PlayGrant { expiry: Expiry::EndOfTurn, any_type: true, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT, steal, "Whenever Black Widow deals combat damage to a player, that player exiles cards from the top of their library until they exile a nonland card. You may put a +1/+1 counter on Black Widow. If you don't, you may cast the exiled nonland card until end of turn and mana of any type can be spent to cast that spell.")],
    ..oracle::BLACK_WIDOW_SUPER_SPY
};
