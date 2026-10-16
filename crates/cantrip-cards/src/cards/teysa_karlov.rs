use cantrip_api::*;
use cantrip_mechanics::statics::{creature_dying_triggers_again, your_creatures};
use cantrip_oracle as oracle;

/// "Creature tokens you control" (CR 111.1, 613.1f): a creature you control that is a token, judged on what the earlier
/// layers made it.
fn creature_tokens_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().obj(affected).token && your_creatures(q, source, affected, so_far)
}

/// "... have vigilance and lifelink." (CR 702.20, 702.15)
fn vigilance_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Vigilance).with(Keyword::Lifelink);
}

/// "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d)
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, creature_tokens_you_control, vigilance_and_lifelink, "Creature tokens you control have vigilance and lifelink.")],
    additional_triggers: &[creature_dying_triggers_again],
    ..oracle::TEYSA_KARLOV
};
