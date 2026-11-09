use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{2}").and(TAP),
                effects::target_creature_gets_while_source_tapped::<2, -2>,
                "{2}, {T}: Target creature you control gets +2/-2 for as long as this artifact remains tapped.",
            )
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    may_not_untap: true,
    ..oracle::ASHNODS_BATTLE_GEAR
};
