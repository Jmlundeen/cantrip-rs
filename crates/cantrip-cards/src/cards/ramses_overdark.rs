use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{T}: Destroy target enchanted creature." (CR 115.1).
fn destroy_target_enchanted_creature_target_0(
    q: &dyn Query,
    _: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if targets::permanent(q, id) && targets::is_enchanted(q, id) && q.chars(id).is(CardType::Creature)
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
