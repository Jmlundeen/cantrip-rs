use cantrip_api::*;
use cantrip_mechanics::{destroy_target, effects, targets};
use cantrip_oracle as oracle;

/// "Target creature with toughness 4 or greater" (CR 115.1): its toughness as the target is chosen.
fn creature_with_toughness_four_or_greater(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if targets::permanent(q, id)
            && q.chars(id).is(CardType::Creature)
            && q.chars(id).toughness.is_some_and(|t| t >= 4))
}

/// "Destroy target creature with toughness 4 or greater. You gain 1 life." (CR 701.8, 608.2b): with its one target illegal
/// the core does not resolve the spell at all, so no life; with a legal target the life is gained whether or not the destroy
/// did anything (an indestructible target, a regeneration shield), the instructions being independent (CR 608.2c).
fn destroy_and_gain_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    effects::gain_life(g, item, 1)
}

/// "You draw a card and gain 2 life."
fn draw_and_gain_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)?;
    effects::gain_life(g, item, 2)
}

static MODAL: ModalDef = modes(&[
    mode(destroy_and_gain_one, "Destroy target creature with toughness 4 or greater. You gain 1 life.")
        .targets(&[target(creature_with_toughness_four_or_greater)]),
    mode(draw_and_gain_two, "You draw a card and gain 2 life."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SURGICAL_PRECISION };
