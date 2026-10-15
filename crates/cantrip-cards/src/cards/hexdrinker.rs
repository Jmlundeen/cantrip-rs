use cantrip_api::*;
use cantrip_mechanics::level_up;
use cantrip_oracle as oracle;

/// LEVEL 3-7 (CR 711.2a): the level symbol's abilities apply while it has that many level counters.
fn level_three(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && (3..=7).contains(&level_up::levels(view, source))
}

/// LEVEL 8+.
fn level_eight(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && level_up::levels(view, source) >= 8
}

fn four_four(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(4);
    chars.toughness = Some(4);
}

fn six_six(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(6);
    chars.toughness = Some(6);
}

fn protection_from_instants(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_types = chars.protection_types.with(CardType::Instant);
}

fn protection_from_everything(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.protection_everything = true;
}

pub static CARD: CardDef = CardDef {
    abilities: &[level_up::ability(mana(1, 0, 0, 0, 0, 0), "Level up {1}")],
    statics: &[
        StaticDef {
            text: "LEVEL 3-7: 4/4",
            layer: Layer::PtSet,
            affects: level_three,
            apply: four_four,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "LEVEL 3-7: Protection from instants",
            layer: Layer::Ability,
            affects: level_three,
            apply: protection_from_instants,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "LEVEL 8+: 6/6",
            layer: Layer::PtSet,
            affects: level_eight,
            apply: six_six,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "LEVEL 8+: Protection from everything",
            layer: Layer::Ability,
            affects: level_eight,
            apply: protection_from_everything,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::HEXDRINKER
};
