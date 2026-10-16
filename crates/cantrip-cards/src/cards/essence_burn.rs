use cantrip_api::*;
use cantrip_mechanics::replacements::{EXILE_INSTEAD_OF_DYING, exile_if_it_would_die};
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Target black or green creature or planeswalker" (CR 115.1, 608.2b): checked again as the spell resolves. The color
/// words are read through the words of Essence Burn's own text (CR 612).
fn black_or_green_creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        let words = q.words(asker.source);
        (c.is(CardType::Creature) || c.is(CardType::Planeswalker))
            && (c.colors.has(words.color(Color::Black)) || c.colors.has(words.color(Color::Green)))
    })
}

/// "Essence Burn deals 5 damage to target black or green creature or planeswalker. If that permanent would die this turn,
/// exile it instead." The replacement is the spell's own effect on that permanent (CR 611.2c, 614.1a), made whether or not
/// the damage is dealt, and before it, so it covers every way the permanent would die this turn (CR 700.4).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        exile_if_it_would_die(g, item.object, 0, permanent);
        damage_target(g, item, 0, 5);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(black_or_green_creature_or_planeswalker)], effect),
    granted_replacements: &[EXILE_INSTEAD_OF_DYING],
    ..oracle::ESSENCE_BURN
};
