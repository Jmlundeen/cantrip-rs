use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return target creature card with mana value 3 or less from your graveyard to the battlefield with an
/// additional +1/+1 counter on it"
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = item.controller;
    g.put_onto_battlefield_with_counters(card, controller, false, &[(CounterKind::PlusOne, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD], effect),
    ways: &[flashback::way(cost("{4}{W}"), "Flashback {4}{W}")],
    ..oracle::GENEROUS_REVIVAL
};
