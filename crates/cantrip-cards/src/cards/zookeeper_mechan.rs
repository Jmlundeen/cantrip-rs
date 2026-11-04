use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::R), "{T}: Add {R}."),
        ability(
            cost("{6}{R}"),
            effects::target_creature_gets::<4, 0>,
            "{6}{R}: Target creature you control gets +4/+0 until end of turn. Activate only as a sorcery.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL])
        .sorcery_speed(),
    ],
    ..oracle::ZOOKEEPER_MECHAN
};
