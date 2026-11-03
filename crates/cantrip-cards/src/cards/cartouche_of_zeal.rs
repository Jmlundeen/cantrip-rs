use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, plus_pt};
use cantrip_mechanics::{attach, effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            effects::first_target_cant_block_this_turn,
            "When this Aura enters, target creature can't block this turn.",
        )
        .targets(&[targets::CREATURE]),
    ],
    statics: &[
        static_ability(Layer::Ability, attach::attached_object, have_haste, "Enchanted creature gets +1/+1 and has haste."),
        static_ability(Layer::PtModify, attach::attached_object, plus_pt::<1, 1>, "Enchanted creature gets +1/+1 and has haste.").continues(),
    ],
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    ..oracle::CARTOUCHE_OF_ZEAL
};
