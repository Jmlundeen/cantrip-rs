use cantrip_api::*;
use cantrip_mechanics::{effects, mana, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        ability(
            cost("{2}{B}").and(SACRIFICE_THIS).and(TAP),
            effects::return_first_target_to_hand,
            "{2}{B}, {T}, Sacrifice this land: Return target creature card from your graveyard to your hand.",
        )
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::MEMORIAL_TO_FOLLY
};
