use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{B}{R}{G}").and(TAP),
                effects::return_first_target_to_hand,
                "{B}{R}{G}, {T}: Return target creature card from your graveyard to your hand.",
            )
            .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
    ],
    ..oracle::ADUN_OAKENSHIELD
};
