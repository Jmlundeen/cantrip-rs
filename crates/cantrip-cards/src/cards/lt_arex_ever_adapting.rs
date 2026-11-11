use cantrip_api::*;
use cantrip_mechanics::{effects};
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Lt. Arex's power and toughness are each equal to the number of differently named lands you control." (CR 604.3,
/// 613.4a): characteristic-defining, so it applies in every zone and reads current land names through the layers. Captain
/// Pike, Fullness of Time prints the same ability.
pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_differently_named_lands_you_control,
        "Lt. Arex's power and toughness are each equal to the number of differently named lands you control.",
    )
    .lost_pt(0, 0)],
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| effects::draw_then_may_put_up_to_lands_tapped(g, item.controller, item.object, 3),
        "When Lt. Arex enters, draw three cards. You may put up to three land cards from your hand onto the battlefield tapped.",
    )],
    ..oracle::LT_AREX_EVER_ADAPTING
};
