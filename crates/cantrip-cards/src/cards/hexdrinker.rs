use cantrip_api::*;
use cantrip_mechanics::statics::base_pt;
use cantrip_mechanics::level_up;
use cantrip_mechanics::protection::from_type_apply;
use cantrip_oracle as oracle;

/// LEVEL 3-7 (CR 711.2a): the level symbol's abilities apply while it has that many level counters.
fn level_three(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && (3..=7).contains(&level_up::levels(view, source))
}

/// LEVEL 8+.
fn level_eight(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && level_up::levels(view, source) >= 8
}

fn protection_from_everything(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_everything = true;
}

pub static CARD: CardDef = CardDef {
    abilities: &[level_up::ability(mana("{1}"), "Level up {1}")],
    statics: &[
        static_ability(Layer::PtSet, level_three, base_pt::<4, 4>, "LEVEL 3-7: 4/4"),
        static_ability(Layer::Ability, level_three, from_type_apply::<{ CardType::Instant as u8 }>, "LEVEL 3-7: Protection from instants"),
        static_ability(Layer::PtSet, level_eight, base_pt::<6, 6>, "LEVEL 8+: 6/6"),
        static_ability(Layer::Ability, level_eight, protection_from_everything, "LEVEL 8+: Protection from everything"),
    ],
    ..oracle::HEXDRINKER
};
