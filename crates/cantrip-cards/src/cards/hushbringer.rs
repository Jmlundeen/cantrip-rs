use cantrip_api::*;
use cantrip_mechanics::statics::{creature_dying, creatures_entering_dont_trigger};
use cantrip_oracle as oracle;

/// "Creatures ... dying don't cause abilities to trigger." (CR 603.2, 700.4): "dies" and "leaves the battlefield" abilities
/// alike, as `statics::creature_dying` judges the dying.
fn creatures_dying(q: &dyn Query, _: &Asker, event: &Event, triggering: &Triggering) -> bool {
    creature_dying(q, event, triggering)
}

/// "Creatures entering or dying don't cause abilities to trigger."
pub static CARD: CardDef = CardDef { trigger_rules: &[creatures_entering_dont_trigger, creatures_dying], ..oracle::HUSHBRINGER };
