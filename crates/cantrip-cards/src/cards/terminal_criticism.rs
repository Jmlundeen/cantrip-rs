use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker that's blue or red" (CR 115.1, 105.2): the colors are read through the words of this
/// spell's text (CR 612), and a permanent of both colors qualifies.
fn blue_or_red_creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    if !targets::permanent(q, id) {
        return false;
    }
    let chars = q.chars(id);
    let words = q.words(asker.source);
    (chars.is(CardType::Creature) || chars.is(CardType::Planeswalker))
        && (chars.colors.has(words.color(Color::Blue)) || chars.colors.has(words.color(Color::Red)))
}

/// "Destroy target creature or planeswalker that's blue or red. You gain 1 life." (CR 701.8, 608.2b): with its one target
/// illegal the core does not resolve the spell at all, so no life; with a legal target the life is gained whether or not the
/// destroy did anything (an indestructible target, a regeneration shield), the instructions being independent (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    effects::gain_life(g, item, 1)
}

pub static CARD: CardDef = CardDef { spell: spell(&[target(blue_or_red_creature_or_planeswalker)], effect), ..oracle::TERMINAL_CRITICISM };
