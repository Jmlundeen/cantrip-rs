use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::of_subtype_you_control_enters;
use cantrip_oracle as oracle;

/// "Whenever a Spider you control enters, draw a card. This ability triggers only once each turn." (CR 603.2, 121.1): any
/// Spider under her controller's control as it enters, herself too were she one, a token too.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, of_subtype_you_control_enters::<{ lands::sub(Subtype::Spider) }>),
        |g, item| effects::draw(g, item, 1),
        "Whenever a Spider you control enters, draw a card. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)],
    ..oracle::MARY_JANE_WATSON
};
