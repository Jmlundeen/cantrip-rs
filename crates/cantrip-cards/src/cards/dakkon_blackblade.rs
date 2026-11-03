use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(
        Layer::PtDefine,
        lands::power_and_toughness_are_lands_you_control,
        "Dakkon Blackblade's power and toughness are each equal to the number of lands you control.",
    ).lost_pt(0, 0)],
    ..oracle::DAKKON_BLACKBLADE
};
