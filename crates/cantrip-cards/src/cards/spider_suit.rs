use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::lands::sub;
use cantrip_mechanics::statics::{also_a, plus_pt};
use cantrip_oracle as oracle;

/// "Equipped creature gets +2/+2 and is a Spider Hero in addition to its other types" (CR 301.5, 205.1b): the two creature types are
/// added together in the type-changing layer (CR 613.1d) and the bonus in the power/toughness layer (CR 613.4c); "Equip {3}" is the
/// keyword (CR 702.6a).
pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{3}"), "Equip {3}")],
    statics: &[
        static_ability(Layer::Type, attached_object, also_a::<{ sub(Subtype::Spider) }>, "Equipped creature is a Spider in addition to its other types."),
        static_ability(Layer::Type, attached_object, also_a::<{ sub(Subtype::Hero) }>, "Equipped creature is a Hero in addition to its other types."),
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
    ],
    ..oracle::SPIDER_SUIT
};
