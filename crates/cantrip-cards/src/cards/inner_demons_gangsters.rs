use cantrip_api::*;
use cantrip_mechanics::effects::this_gets_and_gains_menace;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        discard_cards(1),
        this_gets_and_gains_menace::<1, 0>,
        "Discard a card: This creature gets +1/+0 and gains menace until end of turn. Activate only as a sorcery.",
    )
    .sorcery_speed()],
    ..oracle::INNER_DEMONS_GANGSTERS
};
