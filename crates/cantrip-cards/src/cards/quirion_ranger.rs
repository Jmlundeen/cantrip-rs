use cantrip_api::*;
use cantrip_mechanics::targets::filters::land_type;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        return_to_hand(1, land_type::<{ lands::sub(Subtype::Forest) }>),
        effects::untap_targets,
        "Return a Forest you control to its owner's hand: Untap target creature. Activate only once each turn.")
        .targets(&[targets::CREATURE])
        .once_each_turn()],
    ..oracle::QUIRION_RANGER
};
