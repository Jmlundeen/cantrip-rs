use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::untap_targets, "{T}: Untap another target permanent you control.").targets(&[targets::ANOTHER_PERMANENT_YOU_CONTROL]),
        ability(
            cost("{3}, {T}"),
            effects::tap_targets,
            "Waterbend {3}, {T}: Tap target creature an opponent controls. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
        )
        .waterbend()
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::NORTH_POLE_PATROL
};
