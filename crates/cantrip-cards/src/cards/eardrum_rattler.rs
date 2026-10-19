use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Another target creature you control with power 2 or less" (CR 115.1, 109.5, 208.1): a creature on the battlefield under
/// this ability's controller's control other than the source, with power 2 or less as it is now.
fn another_creature_you_control_with_power_two_or_less(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && id != asker.source
        && q.controller(id) == asker.controller
        && q.chars(id).is(CardType::Creature)
        && q.chars(id).power.is_some_and(|p| p <= 2))
}

/// The creature can't be blocked this turn (CR 509.1b, 611.2).
fn cant_be_blocked(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::CantBeBlocked])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(TAP), cant_be_blocked, "{1}, {T}: Another target creature you control with power 2 or less can't be blocked this turn.")
        .targets(&[target(another_creature_you_control_with_power_two_or_less)])],
    ..oracle::EARDRUM_RATTLER
};
