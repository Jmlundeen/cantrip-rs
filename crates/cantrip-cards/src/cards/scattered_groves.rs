use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This land enters tapped." (CR 614.1d).
fn land_enters_tapped_enters(
    _: &mut dyn Game,
    entering: &mut Entering,
) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

/// "Cycling {2}" (CR 702.29a): discard this card from the hand and pay {2} to draw a card.
fn cycling_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(land_enters_tapped_enters),
    abilities: &[ability(cost("{2}").and(DISCARD_THIS), cycling_2, "Cycling {2}")
        .cycling()
        .from(ZoneKind::Hand)],
    ..oracle::SCATTERED_GROVES
};
