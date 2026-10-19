use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
            |g, item| effects::gain_life(g, item, 1),
            "Whenever another creature you control enters, you gain 1 life.",
        ),
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
    ],
    ..oracle::GREENHOUSE_PROPAGATOR
};
