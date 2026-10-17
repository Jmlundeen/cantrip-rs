use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Threshold \u{2014} You can't cast this spell unless there are seven or more cards in your graveyard" (CR 601.3;
/// Threshold is an ability word, CR 207.2c). Judged for every way this card could be cast, as the cast begins, while the card
/// is still where it is cast from: a Proft cast from the graveyard is one of the cards there, and the ruling asks for seven
/// OTHER cards, so it is not counted.
fn seven_other_cards_in_your_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::graveyard(asker.controller)).iter().filter(|&&card| card != asker.source).count() >= 7
}

/// "Target creature gets -3/-1 until end of turn." Nothing if the target is gone (CR 608.2b).
fn shrink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -3, -1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    cast_only_if: Some(seven_other_cards_in_your_graveyard),
    abilities: &[ability(cost("{B}").and(DISCARD_THIS), shrink, "{B}, Discard this card: Target creature gets -3/-1 until end of turn.")
        .targets(&[targets::CREATURE])
        .from(ZoneKind::Hand)],
    ..oracle::PROFT_SINISTER_MASTERMIND
};
