use cantrip_api::*;
use cantrip_mechanics::{paradigm, targets};
use cantrip_oracle as oracle;

/// "Return target nonland permanent card from your graveyard to the battlefield." The card, if it is still in the graveyard
/// (CR 608.2b, 400.7), enters under the spell's controller's control (CR 400.3, 110.2).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_onto_battlefield(card, item.controller, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT_CARD_IN_YOUR_GRAVEYARD], effect),
    paradigm: true,
    abilities: &[paradigm::TRIGGER],
    ..oracle::RESTORATION_SEMINAR
};
