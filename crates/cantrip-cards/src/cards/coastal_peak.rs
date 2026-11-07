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

/// "Cycling {2}" (CR 608.2, 121.1).
fn cycling_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}").and(DISCARD_THIS), cycling_2, "Cycling {2}")
            .cycling()
            .from(ZoneKind::Hand),
    ],
    enters: Some(land_enters_tapped_enters),
    ..oracle::COASTAL_PEAK
};
