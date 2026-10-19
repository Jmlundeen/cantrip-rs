use cantrip_api::*;
use cantrip_mechanics::destroy_target;
use cantrip_oracle as oracle;

/// "Target green or white creature" (CR 115.1, 105.2): a creature on the battlefield that is green, white, or both. The
/// color words are those of Deathmark's own text (CR 612), so a text-changing effect on it changes them.
fn green_or_white_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && {
        let c = q.chars(id);
        let words = q.words(asker.source);
        c.is(CardType::Creature) && (c.colors.has(words.color(Color::Green)) || c.colors.has(words.color(Color::White)))
    })
}

/// "Destroy target green or white creature." (CR 701.8): regeneration and indestructible apply, since it does not say
/// otherwise.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[target(green_or_white_creature)], effect), ..oracle::DEATHMARK };
