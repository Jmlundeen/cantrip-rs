use cantrip_api::*;
use cantrip_mechanics::{attach, effects, statics, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_UPKEEP,
        |g, item| effects::sacrifice_this_unless_pay(g, item, cost("{1}{U}")),
        "At the beginning of your upkeep, sacrifice this Aura unless you pay {1}{U}.",
    )],
    statics: &[
        control_changing(attach::attached_object, "You control enchanted creature."),
        static_ability(Layer::PtModify, attach::attached_object, statics::plus_pt::<0, 1>, "Enchanted creature gets +0/+1."),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::BINDING_GRASP
};
