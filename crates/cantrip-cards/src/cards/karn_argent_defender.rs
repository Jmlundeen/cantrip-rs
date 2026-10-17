use cantrip_api::*;
use cantrip_mechanics::statics::artifacts_and_creatures_entering_dont_trigger;
use cantrip_oracle as oracle;

/// "Artifacts and creatures entering the battlefield don't cause abilities to trigger." (CR 603.2, 603.6a): Karn is itself an
/// artifact creature, so his own entering is stopped too, and so is anything entering alongside him.
pub static CARD: CardDef = CardDef { trigger_rules: &[artifacts_and_creatures_entering_dont_trigger], ..oracle::KARN_ARGENT_DEFENDER };
