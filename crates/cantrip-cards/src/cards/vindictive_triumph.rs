use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Exile target creature or planeswalker. If that permanent's mana value was 3 or less, return it to the
/// battlefield tapped under your control." Read its battlefield mana value before exile (CR 608.2h), then follow
/// the card into exile (CR 400.7j). An exiled token cannot return (CR 111.8).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let value = g.mana_value(permanent);
    let token = g.obj(permanent).token;
    let exiled = g.exile(permanent);
    if value <= 3 && !token && g.obj(exiled).zone.is(ZoneKind::Exile) {
        let returned = g.put_onto_battlefield(exiled, item.controller, true)?;
        delayed::at_next_end_step(g, item, 0, Target::Object(returned));
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    // "Exile it at the beginning of the next end step." Only the returned object is affected (CR 603.7c).
    delayed: &[delayed_trigger(delayed::exile_subject, "Exile it at the beginning of the next end step.")],
    ..oracle::VINDICTIVE_TRIUMPH
};
