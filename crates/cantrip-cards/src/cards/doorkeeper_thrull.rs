use cantrip_api::*;
use cantrip_mechanics::statics::artifacts_and_creatures_entering_dont_trigger;
use cantrip_oracle as oracle;

/// "Artifacts and creatures entering don't cause abilities to trigger." (CR 603.2, 603.6a)
pub static CARD: CardDef = CardDef { trigger_rules: &[artifacts_and_creatures_entering_dont_trigger], ..oracle::DOORKEEPER_THRULL };
