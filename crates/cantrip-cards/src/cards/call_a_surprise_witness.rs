use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return target creature card with mana value 3 or less from your graveyard to the battlefield. Put a flying counter
/// on it. It's a Spirit in addition to its other types." The card is returned under the spell's controller's control
/// (CR 400.3, 110.2); the counter (CR 122.1b) and the extra type (CR 205.1b, 611.2a: no duration) go to the permanent it
/// became, if it is still there. A Spirit word is read through the spell's text (CR 612).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let arrived = g.put_onto_battlefield(card, item.controller, false)?;
    if targets::permanent(g, arrived) {
        g.add_counters(arrived, CounterKind::Flying, 1);
        g.add_effect(item.object, arrived, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Spirit)), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD], effect),
    ..oracle::CALL_A_SURPRISE_WITNESS
};
