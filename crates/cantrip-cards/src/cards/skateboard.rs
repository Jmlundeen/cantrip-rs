use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Tap target permanent" (CR 701.26a); a target that became illegal is no longer tapped (CR 608.2b).
fn tap_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.tap(permanent);
    }
    Ok(())
}

fn plus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
}

fn gains_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Haste);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this Equipment enters, tap target permanent.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::PERMANENT],
            effect: tap_target,
            ..TriggeredDef::BLANK
        }),
        equip(mana(1, 0, 0, 0, 0, 0), "Equip {1}"),
    ],
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
            apply: gains_haste,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::SKATEBOARD
};
