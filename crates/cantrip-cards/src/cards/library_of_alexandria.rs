use cantrip_api::*;
use cantrip_oracle as oracle;

fn exactly_seven_cards_in_hand(q: &dyn Query, asker: &Asker) -> bool {
    q.state().zone(Zone::hand(asker.controller)).len() == 7
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(TAP, cantrip_mechanics::effects::draw::<1>, "{T}: Draw a card. Activate only if you have exactly seven cards in hand.")
            .condition(exactly_seven_cards_in_hand),
    ],
    ..oracle::LIBRARY_OF_ALEXANDRIA
};
