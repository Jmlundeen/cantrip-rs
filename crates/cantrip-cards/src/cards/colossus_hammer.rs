use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn plus_ten(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 10);
    chars.toughness = chars.toughness.map(|v| v + 10);
}

/// "... and loses flying" (CR 613.1f): layer 6, which takes the keyword away from what the Equipment is attached to.
fn loses_flying(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = Keywords(chars.keywords.0 & !(1 << Keyword::Flying as u8));
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana(8, 0, 0, 0, 0, 0), "Equip {8}")],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +10/+10.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_ten,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature loses flying.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: loses_flying,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::COLOSSUS_HAMMER
};
