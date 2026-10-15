use cantrip_api::*;
use cantrip_mechanics::{attach::{attached_object, equip}, triggers::ward};
use cantrip_oracle as oracle;

fn plus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
}

fn haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Haste);
}

/// The equipped creature has ward {1} (CR 702.21a): the Boots' own delayed ability 0, a triggered ability that the
/// creature controls and that is gone the moment the Boots stop being attached to it.
fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana(1, 0, 0, 0, 0, 0), "Equip {1}")],
    delayed: &[ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)")],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +1/+0.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_one_power,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has haste.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: haste,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has ward {1}.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: grant_ward,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::LAVASPUR_BOOTS
};
