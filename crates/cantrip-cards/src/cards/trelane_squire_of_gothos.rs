use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Vigilance" (CR 702.20) and "Trelane can't be blocked" (CR 509.1b) come from the content keywords, no code.
/// "Ward {2}" (CR 702.21a) is the shared ward trigger: an opponent's spell targeting Trelane is countered unless
/// that player pays {2} as it resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)"),
    ],
    ..oracle::TRELANE_SQUIRE_OF_GOTHOS
};
