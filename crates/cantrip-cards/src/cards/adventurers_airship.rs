use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::effects::loot;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, loot, "Whenever this Vehicle attacks, draw a card, then discard a card."),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::ADVENTURERS_AIRSHIP
};
