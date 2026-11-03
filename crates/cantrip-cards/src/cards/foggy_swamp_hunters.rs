use cantrip_api::*;
use cantrip_oracle as oracle;

/// "As long as you've drawn two or more cards this turn" (CR 611.3a) as a static ability's `StaticDef::affects`: only this
/// creature, and only while its controller has drawn two or more cards this turn. The layers ask again whenever characteristics
/// are computed, so it turns on with the second draw and off as the turn ends.
fn itself_while_you_drew_two_or_more(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && q.history().drawn(q.state().obj(source).controller) >= 2
}

/// "... has lifelink and menace" (CR 702.15, 702.110, layer 6).
fn have_lifelink_and_menace(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Lifelink).with(Keyword::Menace);
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        itself_while_you_drew_two_or_more,
        have_lifelink_and_menace,
        "As long as you've drawn two or more cards this turn, this creature has lifelink and menace.",
    )],
    ..oracle::FOGGY_SWAMP_HUNTERS
};
