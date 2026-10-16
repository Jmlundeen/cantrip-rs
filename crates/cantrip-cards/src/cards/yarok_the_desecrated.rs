use cantrip_api::*;
use cantrip_mechanics::statics::permanent_entering_triggers_again;
use cantrip_oracle as oracle;

/// "If a permanent entering causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d)
pub static CARD: CardDef = CardDef { additional_triggers: &[permanent_entering_triggers_again], ..oracle::YAROK_THE_DESECRATED };
