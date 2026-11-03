use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, combat::one_blocker_at_most, fight, statics::plus_pt, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[triggered(
        THIS_ENTERS,
        fight::attached_creature_fights_first_target,
        "When this Aura enters, enchanted creature fights up to one target creature an opponent controls. (Each deals damage equal to its power to the other.)",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS.optional()])],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<0, 2>, "Enchanted creature gets +0/+2."),
    ],
    blocker_limits: &[one_blocker_at_most],
    ..oracle::MELTSTRIDERS_RESOLVE
};
