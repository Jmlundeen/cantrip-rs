use cantrip_api::*;
use cantrip_mechanics::{targets, text_change};
use cantrip_oracle as oracle;

/// "Change the text of target spell or permanent by replacing all instances of one basic land type with another. (For
/// example, you may change "swampwalk" to "plainswalk." This effect lasts indefinitely.)" (CR 612): the land type and the
/// one it becomes chosen as it resolves (CR 608.2d), never the same (the rulings). A permanent spell's changed text stays with
/// the permanent it becomes (CR 400.7a, the ruling). Nothing if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else { return Ok(()) };
    let words = text_change::basic_land_type(g, item.controller, item.object)?;
    text_change::change(g, item.object, object, words, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL_OR_PERMANENT], effect), ..oracle::MAGICAL_HACK };
