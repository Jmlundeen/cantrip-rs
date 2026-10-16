use cantrip_api::*;
use cantrip_mechanics::statics::creatures_entering_dont_trigger;
use cantrip_oracle as oracle;

/// "Creatures entering don't cause abilities to trigger." (CR 603.2, 603.6a)
pub static CARD: CardDef = CardDef { trigger_rules: &[creatures_entering_dont_trigger], ..oracle::HUSHWING_GRYFF };
