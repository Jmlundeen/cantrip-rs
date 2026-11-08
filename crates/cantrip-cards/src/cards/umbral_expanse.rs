use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "This land enters tapped." (CR 614.1d): the replacement applies as it enters, so a land played on turn 1 is tapped
/// and cannot be tapped for mana that turn. "{T}: Add {W} or {B}" is not written here: as a Plains Swamp it has that
/// mana ability from its basic land types (CR 305.6, 106.4).
fn land_enters_tapped_enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggers::cycling(
        mana("{2}"),
        "Cycling {2} ({2}, Discard this card: Draw a card.)",
    )],
    enters: Some(land_enters_tapped_enters),
    ..oracle::UMBRAL_EXPANSE
};
