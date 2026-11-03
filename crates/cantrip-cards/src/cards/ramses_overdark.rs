use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{T}: Destroy target enchanted creature." (CR 115.1).
fn destroy_target_enchanted_creature_target_0(
    q: &dyn Query,
    _: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && ((q
        .state().battlefield().iter().any(| & a | q.state().obj(a).attached_to ==
        Some(id) && q.chars(a).has_subtype(Subtype::Aura)) && q.chars(id)
        .is(CardType::Creature)))
    )
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                effects::destroy_first_target,
                "{T}: Destroy target enchanted creature.",
            )
            .targets(&[target(destroy_target_enchanted_creature_target_0)]),
    ],
    ..oracle::RAMSES_OVERDARK
};
