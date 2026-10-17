use cantrip_api::*;
use cantrip_mechanics::{targets, text_change};
use cantrip_oracle as oracle;

/// "Change the text of target spell or permanent by replacing all instances of one creature type with another. The new
/// creature type can't be Wall. (This effect lasts indefinitely.)" (CR 612): any creature type, then any other but Wall,
/// chosen as it resolves (CR 608.2d). A permanent spell's changed text stays with the permanent it becomes (CR 400.7a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else { return Ok(()) };
    let from = text_change::one_subtype(g, item.controller, item.object, Subtype::CREATURE_TYPES.to_vec(), WordUse::Replaced)?;
    let to: Vec<Subtype> = Subtype::CREATURE_TYPES.into_iter().filter(|&s| s != from && s != Subtype::Wall).collect();
    let to = text_change::one_subtype(g, item.controller, item.object, to, WordUse::Replacing)?;
    text_change::change(g, item.object, object, (Word::Subtype(from), Word::Subtype(to)), Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL_OR_PERMANENT], effect),
    ..oracle::ARTIFICIAL_EVOLUTION
};
