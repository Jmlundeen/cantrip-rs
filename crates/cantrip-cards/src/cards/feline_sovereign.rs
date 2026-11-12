use cantrip_api::*;
use cantrip_mechanics::protection::from_subtype_apply;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_mechanics::triggers::subtype_you_control_deal_combat_damage_to_player;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::PtModify,
            other_of_subtype_you_control::<{ lands::sub(Subtype::Cat) }>,
            plus_pt::<1, 1>,
            "Other Cats you control get +1/+1.",
        ),
        static_ability(
            Layer::Ability,
            other_of_subtype_you_control::<{ lands::sub(Subtype::Cat) }>,
            from_subtype_apply::<{ lands::sub(Subtype::Dog) }>,
            "Other Cats you control have protection from Dogs.",
        ),
    ],
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, subtype_you_control_deal_combat_damage_to_player::<{ lands::sub(Subtype::Cat) }>),
        effects::destroy_first_target,
        "Whenever one or more Cats you control deal combat damage to a player, destroy up to one target artifact or enchantment that player controls.",
    )
    .sums_per_subject()
    .targets(&[up_to_one(targets::artifact_or_enchantment_that_player_controls)])],
    ..oracle::FELINE_SOVEREIGN
};
