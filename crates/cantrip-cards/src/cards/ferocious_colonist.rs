use cantrip_api::*;
use cantrip_mechanics::melee;
use cantrip_oracle as oracle;

/// "Double strike" (CR 702.4) and "Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each opponent
/// you attacked this combat.)" (CR 702.121a): the first is a core keyword the content JSON carries and the second is the
/// shared melee trigger, so the card adds no lines of its own.
pub static CARD: CardDef = CardDef {
    abilities: &[melee::TRIGGER],
    ..oracle::FEROCIOUS_COLONIST
};
