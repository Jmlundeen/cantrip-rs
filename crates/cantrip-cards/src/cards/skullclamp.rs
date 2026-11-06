use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana("{1}"), "Equip {1}"),
        triggered(cantrip_mechanics::triggers::ATTACHED_CREATURE_DIES, |g, item| effects::draw(g, item, 2), "Whenever equipped creature dies, draw two cards."),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, -1>, "Equipped creature gets +1/-1.")],
    ..oracle::SKULLCLAMP
};
