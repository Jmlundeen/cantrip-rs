use cantrip_api::*;
use cantrip_mechanics::{attach, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            attach::attached_object,
            cantrip_mechanics::statics::grant_delayed_ability,
            "Enchanted land has \"{T}: Target creature gets +2/+1 until end of turn.\"",
        ),
    ],
    delayed: &[
        ability(TAP, effects::target_creature_gets::<2, 1>, "{T}: Target creature gets +2/+1 until end of turn.")
            .targets(&[targets::CREATURE]),
    ],
    enchant: Some(targets::LAND),
    ..oracle::FORBIDDEN_LORE
};
