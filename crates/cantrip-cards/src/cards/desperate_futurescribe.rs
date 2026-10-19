use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_oracle as oracle;

/// "If you've scried or surveilled this turn" (CR 701.22a, 701.25a): this turn's event history, by the ability's controller.
fn scried_or_surveilled_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history()
        .select(EventQuery::kind(EventMask(EventMask::SCRIED.0 | EventMask::SURVEILLED.0)).this_turn())
        .any(|r| matches!(r.event, Event::Scried { player: p, .. } | Event::Surveilled { player: p, .. } if p == player))
}

/// "+1/+1 until end of turn", or a +1/+1 counter on that creature instead when the condition holds (CR 614.1a: "instead"
/// is a replacement, read as the ability resolves).
fn boost(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    if scried_or_surveilled_this_turn(g, item.controller) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    } else {
        pump(g, item.object, creature, 1, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(BEGINNING_OF_YOUR_COMBAT, boost, "At the beginning of combat on your turn, another target creature you control gets +1/+1 until end of turn. If you've scried or surveilled this turn, put a +1/+1 counter on that creature instead.")
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::DESPERATE_FUTURESCRIBE
};
