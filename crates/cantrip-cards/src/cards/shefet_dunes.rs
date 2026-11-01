use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add(Mana::W), "{T}, Pay 1 life: Add {W}."),
        ability(
            cost("{2}{W}{W}, {T}").and(sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Desert) }>)),
            effects::creatures_you_control_get::<1, 1>,
            "{2}{W}{W}, {T}, Sacrifice a Desert: Creatures you control get +1/+1 until end of turn. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::SHEFET_DUNES
};
