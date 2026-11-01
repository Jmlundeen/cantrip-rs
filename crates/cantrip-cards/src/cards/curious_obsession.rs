use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::effects::{may_draw_a_card, sacrifice_self};
use cantrip_mechanics::history::you_didnt_attack_with_a_creature;
use cantrip_mechanics::{statics, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        sacrifice_self,
        "At the beginning of your end step, if you didn't attack with a creature this turn, sacrifice this Aura.",
    )
    .condition(you_didnt_attack_with_a_creature)],
    delayed: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER,
        may_draw_a_card,
        "Whenever this creature deals combat damage to a player, you may draw a card.",
    )],
    statics: &[
        static_ability(Layer::PtModify, attached_object, statics::plus_pt::<1, 1>, "Enchanted creature gets +1/+1."),
        static_ability(
            Layer::Ability,
            attached_object,
            statics::grant_delayed_ability,
            "Enchanted creature has \"Whenever this creature deals combat damage to a player, you may draw a card.\"",
        ),
    ],
    ..oracle::CURIOUS_OBSESSION
};
