use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::EQUIPPED_CREATURE_ATTACKS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(EQUIPPED_CREATURE_ATTACKS, effects::destroy_first_target, "Whenever equipped creature attacks, destroy target creature an opponent controls.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        equip(mana("{7}"), "Equip {7}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<7, 7>, "Equipped creature gets +7/+7.")],
    ..oracle::ULTIMA_WEAPON
};
