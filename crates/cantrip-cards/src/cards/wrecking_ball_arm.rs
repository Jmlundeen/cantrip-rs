use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_oracle as oracle;

/// The equipped creature: what the Equipment is attached to while it is still that creature (CR 301.5a).
fn equipped_creature(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    attached_object(view, source, affected, so_far) && view.chars(affected).is(CardType::Creature)
}

/// Base power and toughness 7/7 (CR 613.4b), below every modifier and counter.
fn size(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = Some(7);
    chars.toughness = Some(7);
}

/// An ability the creature gains in layer 6 (CR 613.1f), so losing all abilities takes it away.
fn evasion(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::CantBeBlockedByPowerTwoOrLess);
}

fn legendary_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Creature) && c.supertypes.legendary
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{3}"), attach_to_target, "Equip legendary creature {3}")
            .sorcery_speed()
            .targets(&[TargetSpec { min: 1, max: 1, filter: legendary_creature_you_control }]),
        equip(mana(7, 0, 0, 0, 0, 0), "Equip {7}"),
    ],
    statics: &[
        StaticDef {
            text: "Equipped creature has base power and toughness 7/7.",
            layer: Layer::PtSet,
            affects: equipped_creature,
            apply: size,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature can't be blocked by creatures with power 2 or less.",
            layer: Layer::Ability,
            affects: equipped_creature,
            apply: evasion,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::WRECKING_BALL_ARM
};
