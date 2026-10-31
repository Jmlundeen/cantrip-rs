use cantrip_api::*;
use cantrip_mechanics::{effects, mana as mana_effects, targets::filters::creature};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana_effects::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Green, Color::White, Color::Blue]), "{T}: Add {G}, {W}, or {U}."),
        ability(
            Cost::mana(mana("{2}")).and(return_tapped_to_hand(1, creature)),
            effects::return_this_card_to_battlefield,
            "{2}, Return a tapped creature you control to its owner's hand: Put this card from your hand onto the battlefield. Activate only as a sorcery.",
        )
        .from(ZoneKind::Hand)
        .sorcery_speed(),
    ],
    ..oracle::URBAN_RETREAT
};
