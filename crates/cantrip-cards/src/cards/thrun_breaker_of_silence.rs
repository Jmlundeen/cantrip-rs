use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Thrun can't be the target of ...": the static ability names Thrun alone (CR 604.1, 201.5).
fn thrun(_: &dyn LayerQuery, source: ObjectId, subject: ObjectId) -> bool {
    source == subject
}

/// "... nongreen spells your opponents control or abilities from nongreen sources your opponents control" (CR 601.2c,
/// 602.2b, 603.3d, 608.2b): a spell or ability an opponent of Thrun's controller controls whose source - the spell itself,
/// or the ability's source as it is or last was - is not green. Colorless is nongreen (CR 105.2c). "Green" is a word of
/// Thrun's text (CR 612.1).
fn nongreen_from_an_opponent(q: &dyn LayerQuery, source: ObjectId, by: &Asker, by_source: &Chars) -> bool {
    by.controller != q.controller(source) && !by_source.colors.has(q.words(source).color(Color::Green))
}

pub static CARD: CardDef = CardDef {
    targeting_rules: &[TargetingRule { kind: TargetingRuleKind::Forbid(nongreen_from_an_opponent), applies: thrun }],
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_indestructible, "During your turn, Thrun has indestructible.")],
    ..oracle::THRUN_BREAKER_OF_SILENCE
};
