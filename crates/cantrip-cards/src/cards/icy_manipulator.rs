use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{1}, {T}: Tap target artifact, creature, or land." (CR 115.1).
fn tap_target_artifact_creature_land_target_0(
    q: &dyn Query,
    _: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if targets::permanent(q, id) && ((q
        .chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Creature) || q
        .chars(id).is(CardType::Land)))
    )
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{1}").and(TAP),
                effects::tap_targets,
                "{1}, {T}: Tap target artifact, creature, or land.",
            )
            .targets(&[target(tap_target_artifact_creature_land_target_0)]),
    ],
    ..oracle::ICY_MANIPULATOR
};
