use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            TAP,
            effects::tap_target_it_doesnt_untap_while_this_remains_tapped,
            "{T}: Tap target creature without flying that's attacking you. It doesn't untap during its controller's untap step for as long as this land remains tapped.",
        )
        .targets(&[targets::CREATURE_WITHOUT_FLYING_ATTACKING_YOU]),
    ],
    may_not_untap: true,
    ..oracle::ICE_FLOE
};
