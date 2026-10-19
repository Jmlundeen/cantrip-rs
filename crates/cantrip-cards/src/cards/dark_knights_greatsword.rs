use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip_cost};
use cantrip_mechanics::job_select::{HERO, JOB_SELECT};
use cantrip_oracle as oracle;

/// "Equipped creature is a Knight in addition to its other types" (layer 4); Knight is a word of this card's text (CR 612.2).
fn knight(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(view.words(source).subtype(Subtype::Knight));
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        JOB_SELECT,
        // "Chaosbringer \u{2014} Equip\u{2014}Pay 3 life" (CR 702.6a): an equip ability whose cost is life, once each turn.
        equip_cost(life(3), "Chaosbringer \u{2014} Equip\u{2014}Pay 3 life. Activate only once each turn.").once_each_turn(),
    ],
    statics: &[
        static_ability(Layer::Type, attached_object, knight, "Equipped creature is a Knight in addition to its other types."),
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<3, 0>, "Equipped creature gets +3/+0."),
    ],
    ..oracle::DARK_KNIGHTS_GREATSWORD
};

pub static EXTRA: &[&CardDef] = &[&HERO];
