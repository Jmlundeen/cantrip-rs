use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "You have shroud. (You can't be the target of spells or abilities.)" (CR 702.18a): its controller, while it is on the
/// battlefield with its abilities, can't be the target of any spell or ability, their own included. Effects that don't
/// target ("each player", "each opponent") still reach them.
pub static CARD: CardDef = CardDef { player_keywords: &[statics::YOU_HAVE_SHROUD], ..oracle::TRUE_BELIEVER };
