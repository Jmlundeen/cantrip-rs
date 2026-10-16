use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "... and loses flying" (CR 613.1f): layer 6, which takes the keyword away from what the Equipment is attached to.
fn loses_flying(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = Keywords(chars.keywords.0 & !(1 << Keyword::Flying as u8));
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{8}"), "Equip {8}")],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<10, 10>, "Equipped creature gets +10/+10."),
        static_ability(Layer::Ability, attached_object, loses_flying, "Equipped creature loses flying."),
    ],
    ..oracle::COLOSSUS_HAMMER
};
