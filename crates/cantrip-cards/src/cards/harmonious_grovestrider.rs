use cantrip_api::*;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

/// "Harmonious Grovestrider's power and toughness are each equal to the number of lands you control" (CR 604.3, layer 7a): a
/// characteristic-defining ability, so it works in every zone, counting its owner's lands off the battlefield.
pub static CARD: CardDef = CardDef {
    abilities: &[triggers::ward::<2>(
        "Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)",
    )],
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_lands_you_control,
        "Harmonious Grovestrider's power and toughness are each equal to the number of lands you control.",
    )
    .lost_pt(0, 0)],
    ..oracle::HARMONIOUS_GROVESTRIDER
};
