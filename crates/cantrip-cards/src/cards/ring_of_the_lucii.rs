use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        ability(cost("{2}").and(TAP).and(life(1)), effects::tap_targets, "{2}, {T}, Pay 1 life: Tap target nonland permanent.")
            .targets(&[targets::NONLAND_PERMANENT]),
    ],
    ..oracle::RING_OF_THE_LUCII
};
