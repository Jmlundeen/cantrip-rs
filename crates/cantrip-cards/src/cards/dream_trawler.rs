use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOU_DRAW_A_CARD,
            effects::this_gets::<1, 0>,
            "Whenever you draw a card, this creature gets +1/+0 until end of turn.",
        ),
        triggered(
            triggers::THIS_ATTACKS,
            |g, item| effects::draw(g, item, 1),
            "Whenever this creature attacks, draw a card.",
        ),
        ability(
            discard_cards(1),
            effects::this_gains_hexproof_then_tap,
            "Discard a card: This creature gains hexproof until end of turn. Tap it.",
        ),
    ],
    ..oracle::DREAM_TRAWLER
};
