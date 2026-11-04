use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::another_of_subtype_you_control_enters;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// Menace is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Villain) }>),
        effects::this_connives,
        "Whenever another Villain you control enters, Prowler connives.",
    )],
    ..oracle::PROWLER_CLAWED_THIEF
};
