use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::ATTACHED_CREATURE_DIES, |g, item| effects::draw(g, item, 1), "Whenever equipped creature dies, draw a card."),
        equip(mana("{2}"), "Equip {2}"),
        equip(mana("{B}"), "Equip {B}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<2, 0>, "Equipped creature gets +2/+0.")],
    ..oracle::TRANSMOGRANTS_CROWN
};
