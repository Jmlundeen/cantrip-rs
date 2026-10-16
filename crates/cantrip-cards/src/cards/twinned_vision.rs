use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Draw a card. If this spell wasn't cast from your hand, draw two cards instead." Where it was cast from is the cast
/// record's zone (CR 601.2a); a copy was never cast, so it is not "cast from your hand" and draws two.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let from_hand = g.history().cast_of(item.object).is_some_and(|c| c.from == ZoneKind::Hand);
    g.draw(item.controller, if from_hand { 1 } else { 2 });
    Ok(())
}

/// "Flashback—{1}{U/R}{U/R}, Discard a card." (CR 702.34a): the discard is an additional cost of this way (CR 118.8).
pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{1}{U/R}{U/R}").and(discard(1, targets::filters::any)), "Flashback\u{2014}{1}{U/R}{U/R}, Discard a card")],
    ..oracle::TWINNED_VISION
};
