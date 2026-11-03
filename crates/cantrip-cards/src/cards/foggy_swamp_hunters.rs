use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "... has lifelink and menace" (CR 702.15, 702.111, layer 6).
fn have_lifelink_and_menace(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink).with(Keyword::Menace);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        statics::itself_while_you_drew_two_or_more,
        have_lifelink_and_menace,
        "As long as you've drawn two or more cards this turn, this creature has lifelink and menace.",
    )],
    ..oracle::FOGGY_SWAMP_HUNTERS
};
