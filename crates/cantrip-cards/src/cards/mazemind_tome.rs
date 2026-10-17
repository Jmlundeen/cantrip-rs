use cantrip_api::*;
use cantrip_mechanics::scry;
use cantrip_oracle as oracle;

/// "{T}, Put a page counter on this artifact: Scry 1."
fn scry_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 1)
}

/// "{2}, {T}, Put a page counter on this artifact: Draw a card."
fn draw_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "When there are four or more page counters on this artifact": a state trigger (CR 603.8), true of the Tome itself.
fn four_or_more_pages(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).counters(CounterKind::Page) >= 4
}

/// "... exile it. If you do, you gain 4 life." Only the Tome that triggered it can be exiled: once it has left the
/// battlefield it is a new object, and nothing is exiled and no life gained (CR 400.7, the ruling). "If you do" is that it
/// was exiled (CR 608.2c): a replacement that sends it elsewhere instead leaves the life ungained.
fn exile_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exile_all(&[item.object]);
    if exiled.iter().any(|&id| g.obj(id).zone.is(ZoneKind::Exile)) {
        g.gain_life(item.controller, 4);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{T}").and(put_counters(CounterKind::Page, 1)), scry_one, "{T}, Put a page counter on this artifact: Scry 1."),
        ability(cost("{2}, {T}").and(put_counters(CounterKind::Page, 1)), draw_one, "{2}, {T}, Put a page counter on this artifact: Draw a card."),
        state_triggered(four_or_more_pages, exile_and_gain, "When there are four or more page counters on this artifact, exile it. If you do, you gain 4 life."),
    ],
    ..oracle::MAZEMIND_TOME
};
