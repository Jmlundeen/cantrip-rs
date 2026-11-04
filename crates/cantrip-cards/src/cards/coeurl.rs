use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// A creature that is not also an enchantment, as the layers have it (CR 115.1).
fn nonenchantment_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::creature_filter(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).is(CardType::Enchantment))
}

/// "{1}{W}, {T}: Tap target nonenchantment creature." (CR 602.2, 701.26a).
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{W}").and(TAP), effects::tap_targets, "{1}{W}, {T}: Tap target nonenchantment creature.")
        .targets(&[target(nonenchantment_creature)])],
    ..oracle::COEURL
};
