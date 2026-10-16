use cantrip_api::*;
use cantrip_mechanics::{exile_target, scry, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker that's black or red" (CR 115.1, 105.1): the color words are read through the words of
/// the spell (CR 612), and the target is checked again as the spell resolves (CR 608.2b).
fn black_or_red_creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    if !(targets::CREATURE_OR_PLANESWALKER.filter)(q, asker, t) {
        return false;
    }
    let Target::Object(id) = t else { return false };
    let words = q.words(asker.source);
    let colors = q.chars(id).colors;
    colors.has(words.color(Color::Black)) || colors.has(words.color(Color::Red))
}

/// "Exile target creature or planeswalker that's black or red. Scry 1." The scry is part of the same effect, so a spell
/// whose only target is illegal on resolution does neither (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    scry(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(black_or_red_creature_or_planeswalker)], effect),
    ..oracle::DEVOUT_DECREE
};
