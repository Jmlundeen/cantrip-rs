use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Another target creature or planeswalker you control" (CR 115.1): a permanent on the battlefield, not Liliana.
fn another_creature_or_planeswalker_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source
        && targets::permanent(q, id)
        && q.controller(id) == asker.controller
        && { let c = q.chars(id); c.is(CardType::Creature) || c.is(CardType::Planeswalker) })
}

/// The target gains hexproof until end of turn (CR 702.11b, 611.2a); nothing happens if it is no longer legal (CR 608.2b).
fn hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        cantrip_mechanics::effects::gain_hexproof_until_end_of_turn(g, item.object, permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, triggers::another_creature_or_planeswalker_you_control_enters), |g, item| { g.gain_life(item.controller, 1); Ok(()) }, "Whenever another creature or planeswalker you control enters, you gain 1 life."),
        ability(cost("{1}, {T}").and(discard_cards(1)), hexproof, "{1}, {T}, Discard a card: Another target creature or planeswalker you control gains hexproof until end of turn.")
            .targets(&[target(another_creature_or_planeswalker_you_control)]),
    ],
    ..oracle::LILIANA_THE_FAULTLESS
};
