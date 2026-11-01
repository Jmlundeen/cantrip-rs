use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

/// "Another target Ally you control" (CR 115.1, 205.3m, 702.73a): any Ally permanent under your control, not only a creature
/// (Firdoch Core is a Kindred Artifact with changeling), other than the source.
fn another_ally_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_subtype_you_control::<{ lands::sub(Subtype::Ally) }>(q, asker, t) && t != Target::Object(asker.source)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        SACRIFICE_THIS,
        effects::target_creature_gains_indestructible,
        "Sacrifice this creature: Another target Ally you control gains indestructible until end of turn.",
    )
    .targets(&[target(another_ally_you_control)])],
    ..oracle::EARTH_KINGDOM_PROTECTORS
};
