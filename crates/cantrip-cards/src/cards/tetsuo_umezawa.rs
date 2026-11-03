use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{U}{B}{B}{R}, {T}: Destroy target tapped or blocking creature." (CR 115.1).
fn tapped_or_blocking_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::creature_filter(q, asker, t)
        && matches!(t, Target::Object(id) if q.obj(id).tapped || q.is_blocking(id))
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{U}{B}{B}{R}").and(TAP),
        effects::destroy_first_target,
        "{U}{B}{B}{R}, {T}: Destroy target tapped or blocking creature.",
    )
    .targets(&[target(tapped_or_blocking_creature)])],
    targeting_rules: &[targets::CANT_BE_TARGET_OF_AURA_SPELLS],
    ..oracle::TETSUO_UMEZAWA
};
