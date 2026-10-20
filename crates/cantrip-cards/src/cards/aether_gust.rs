use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Target spell or permanent that's red or green" (CR 115.1, 105.2): a spell on the stack or a permanent with either
/// color, or both. The color words are those of Aether Gust's own text (CR 612).
fn red_or_green_spell_or_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    if !(targets::SPELL_OR_PERMANENT.filter)(q, asker, t) {
        return false;
    }
    let Target::Object(id) = t else { return false };
    let words = q.words(asker.source);
    let colors = q.chars(id).colors;
    colors.has(words.color(Color::Red)) || colors.has(words.color(Color::Green))
}

/// "Choose target spell or permanent that's red or green. Its owner puts it on their choice of the top or bottom of
/// their library." The owner (CR 108.3), not the controller, chooses. A spell leaves the stack without being countered, so
/// one that can't be countered goes too. With its target gone the spell does not resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else { return Ok(()) };
    library::owner_puts_on_top_or_bottom(g, item.object, object)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(red_or_green_spell_or_permanent)], effect),
    ..oracle::AETHER_GUST
};
