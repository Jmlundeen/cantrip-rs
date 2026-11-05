use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +1/-1 until end of turn. When it dies under your control this turn, draw two cards." The delayed
/// triggered ability (CR 603.7c) is fixed on that creature, so it triggers only when that very object dies (a creature that
/// left and came back is a new object, CR 400.7).
fn desperate_measures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, creature, 1, -1);
    delayed::this_turn(g, item, creature);
    Ok(())
}

/// "When it dies under your control" (CR 700.4, 603.10a): this creature, as it last was on the battlefield, under the
/// controller it last had, who is this ability's controller (CR 603.7d-e).
fn it_dies_under_your_control(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if *old == me.source && from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && q.obj(*old).controller == me.controller =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Draw two cards", by the spell's controller.
fn draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::draw(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], desperate_measures),
    delayed: &[triggered(when_asked(EventMask::ZONE_CHANGE, it_dies_under_your_control), draw_two, "When it dies under your control this turn, draw two cards.")],
    ..oracle::DESPERATE_MEASURES
};
