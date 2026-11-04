use cantrip_api::*;
use cantrip_mechanics::{mayhem, targets};
use cantrip_oracle as oracle;

/// "Return target creature card from your graveyard to the battlefield. It enters with an additional +1/+1 counter on it."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::PlusOne, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD], effect),
    ways: &[mayhem::way(mana("{3}{B}"), "Mayhem {3}{B}")],
    ..oracle::PRISON_BREAK
};
