use cantrip_api::*;
use cantrip_mechanics::statics::creature_attacking_triggers_again;
use cantrip_oracle as oracle;

/// "If a creature attacking causes a triggered ability of a permanent you control to trigger, that ability triggers an
/// additional time." (CR 603.2d, 508.2a)
pub static CARD: CardDef = CardDef { additional_triggers: &[creature_attacking_triggers_again], ..oracle::ISSHIN_TWO_HEAVENS_AS_ONE };
