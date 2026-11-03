use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets, your_upkeep};
use cantrip_oracle as oracle;

/// Put a matrix counter on the target and grant it the regeneration ability (CR 608.2c, 122.1).
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let ability = AbilityRef::delayed(item.ability.card, 0);
    instruction(g, |g| {
        g.add_counters(creature, CounterKind::Matrix, 1);
        g.add_effect(item.object, creature, EffectKind::AddAbility(ability), Duration::Indefinite);
    });
    Ok(())
}

/// The granted ability regenerates its source (CR 701.19a).
fn regenerate_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::this(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{4}").and(TAP),
        grant,
        "{4}, {T}: Put a matrix counter on target creature and that creature gains \"Remove a matrix counter from this creature: Regenerate this creature.\" Activate only during your upkeep.",
    ).targets(&[targets::CREATURE]).condition(your_upkeep)],
    delayed: &[ability(remove_counters(CounterKind::Matrix, 1), regenerate_this,
        "Remove a matrix counter from this creature: Regenerate this creature.")],
    ..oracle::LIFE_MATRIX
};
