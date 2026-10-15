use cantrip_api::*;
use cantrip_mechanics::{cipher, targets, text_change};
use cantrip_oracle as oracle;

/// "Change the text of target permanent by replacing all instances of one color word with another or one basic land type
/// with another until end of turn" (CR 612), the words chosen as it resolves (CR 608.2d); then cipher (CR 702.99a). A
/// target gone illegal leaves the spell with no target: it does nothing, cipher included (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let words = text_change::color_word_or_basic_land_type(g, item.controller, item.object)?;
    text_change::change(g, item.object, permanent, words, Duration::EndOfTurn);
    cipher::encode(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PERMANENT], effect }),
    labels: text_change::LABELS,
    statics: &[cipher::STATIC],
    delayed: &[cipher::TRIGGER],
    ..oracle::TRAIT_DOCTORING
};
