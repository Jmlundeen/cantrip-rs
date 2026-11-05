use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Return target permanent card from your graveyard to the battlefield with a hexproof counter and an indestructible counter on it."
/// The counters are on it as it enters (CR 122.6, 614.1c); with the card gone from the graveyard, nothing happens (CR 608.2b).
fn return_with_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::Hexproof, 1), (CounterKind::Indestructible, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD], return_with_counters),
    ..oracle::PERENNATION
};
