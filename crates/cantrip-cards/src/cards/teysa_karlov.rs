use cantrip_api::*;
use cantrip_mechanics::statics::{creature_dying_triggers_again, have_vigilance, your_creature_tokens};
use cantrip_oracle as oracle;

/// "... have vigilance and lifelink." (CR 702.20, 702.15)
fn vigilance_and_lifelink(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    have_vigilance(view, source, chars);
    chars.keywords = chars.keywords.with(Keyword::Lifelink);
}

/// "If a creature dying causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d)
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, your_creature_tokens, vigilance_and_lifelink, "Creature tokens you control have vigilance and lifelink.")],
    additional_triggers: &[creature_dying_triggers_again],
    ..oracle::TEYSA_KARLOV
};
