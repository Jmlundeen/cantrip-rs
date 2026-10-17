use cantrip_api::*;
use cantrip_mechanics::{targets, text_change};
use cantrip_oracle as oracle;

/// "Change the text of target permanent by replacing all instances of one color word with another or one basic land type with
/// another. (... This effect lasts indefinitely.)" (CR 612): the kind of word, the word and the one it becomes are chosen as
/// it resolves (CR 608.2d), and a word can't become itself (the rulings). Nothing if the target is no longer legal (CR
/// 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let words = text_change::color_word_or_basic_land_type(g, item.controller, item.object)?;
    text_change::change(g, item.object, permanent, words, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PERMANENT], effect), labels: text_change::LABELS, ..oracle::MIND_BEND };
