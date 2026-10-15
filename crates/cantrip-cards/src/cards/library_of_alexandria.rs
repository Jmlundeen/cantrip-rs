use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

fn exactly_seven_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).len() == 7
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Draw a card. Activate only if you have exactly seven cards in hand.",
            cost: Cost { tap: true, ..Cost::NONE },
            condition: Some(exactly_seven_cards_in_hand),
            effect: cantrip_mechanics::effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::LIBRARY_OF_ALEXANDRIA
};
