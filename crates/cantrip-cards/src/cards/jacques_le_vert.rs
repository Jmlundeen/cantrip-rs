use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// Green creatures you control, using the color word in this object's text (CR 612.2, 613.4c).
fn green_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    statics::your_creatures(q, source, affected, chars) && chars.colors.has(q.words(source).color(Color::Green))
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        green_creatures_you_control,
        statics::plus_pt::<0, 2>,
        "Green creatures you control get +0/+2.",
    )],
    ..oracle::JACQUES_LE_VERT
};
