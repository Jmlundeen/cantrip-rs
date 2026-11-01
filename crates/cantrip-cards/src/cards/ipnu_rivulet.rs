use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add(Mana::U), "{T}, Pay 1 life: Add {U}."),
        ability(
            cost("{1}{U}, {T}").and(sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Desert) }>)),
            effects::target_player_mills::<4>,
            "{1}{U}, {T}, Sacrifice a Desert: Target player mills four cards.",
        )
        .targets(&[targets::PLAYER]),
    ],
    ..oracle::IPNU_RIVULET
};
