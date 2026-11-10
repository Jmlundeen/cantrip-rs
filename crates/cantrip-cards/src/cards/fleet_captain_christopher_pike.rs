use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Yes — {1}, {T}: Untap two other target creatures." and "No — {2}, {T}: Tap two other target creatures." Each is one
/// slot of exactly two targets, and "other" leaves out Pike himself (CR 115.1b): both targets are chosen as the ability is
/// activated, and each that is still legal is untapped or tapped together as it resolves (CR 701.26a, 608.2b).
pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}").and(TAP), effects::untap_targets, "Yes \u{2014} {1}, {T}: Untap two other target creatures.")
            .targets(&[targets::OTHER_CREATURE.between(2, 2)]),
        ability(cost("{2}").and(TAP), effects::tap_targets, "No \u{2014} {2}, {T}: Tap two other target creatures.")
            .targets(&[targets::OTHER_CREATURE.between(2, 2)]),
    ],
    ..oracle::FLEET_CAPTAIN_CHRISTOPHER_PIKE
};
