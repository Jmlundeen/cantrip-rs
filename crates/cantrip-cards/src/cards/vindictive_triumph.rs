use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or planeswalker. If that permanent's mana value was 3 or less, return it to the
/// battlefield tapped under your control." Read its battlefield mana value before exile (CR 608.2h), then follow
/// the card into exile (CR 400.7j). An exiled token cannot return (CR 111.8). Each card of a merged permanent (the permanent's
/// one mana value, CR 730.3c) returns, together, and the delayed ability exiles each that did.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let value = g.mana_value(permanent);
    let cards: Vec<(ObjectId, PlayerId, bool)> = g
        .exile_all(&[permanent])
        .into_iter()
        .filter(|&c| !g.obj(c).token && g.obj(c).zone.is(ZoneKind::Exile))
        .map(|c| (c, item.controller, true))
        .collect();
    if value <= 3 && !cards.is_empty() {
        let returned = g.put_all_onto_battlefield_as(&cards)?;
        delayed::at_next_end_step_about(g, item, 0, &returned);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    // "Exile it at the beginning of the next end step." Only the returned object is affected (CR 603.7c).
    delayed: &[delayed_trigger(delayed::exile_subject, "Exile it at the beginning of the next end step.")],
    ..oracle::VINDICTIVE_TRIUMPH
};
