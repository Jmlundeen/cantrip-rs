use cantrip_api::*;
use cantrip_mechanics::statics::{every_basic_land_type, your_lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // "You may play an additional land on each of your turns." (CR 305.2: cumulative with other such effects, the ruling.)
    play_rules: &[PlayRule::ExtraLandPlays(1)],
    // "Lands you control are every basic land type in addition to their other types." (CR 305.7, 613.1d, layer 4)
    statics: &[static_ability(
        Layer::Type,
        your_lands,
        every_basic_land_type,
        "Lands you control are every basic land type in addition to their other types.",
    )],
    ..oracle::DRYAD_OF_THE_ILYSIAN_GROVE
};
