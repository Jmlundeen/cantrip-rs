use cantrip_api::*;
use cantrip_mechanics::combat;
use cantrip_oracle as oracle;

/// Vigilance is a core keyword (CR 702.20), from the card's characteristics.
///
/// "Each creature you control with toughness greater than its power assigns combat damage equal to its toughness rather
/// than its power" (CR 510.1a), judged as combat damage is assigned; and "Each creature you control with toughness greater
/// than its power stations permanents using its toughness rather than its power" (CR 702.184c), judged as the station
/// ability resolves, of the tapped creature as it last existed if it has left (the rulings). Neither changes its power.
pub static CARD: CardDef = CardDef {
    damage_by_toughness: &[combat::your_creature_with_toughness_greater_than_power],
    stations_by_toughness: &[combat::your_creature_with_toughness_greater_than_power],
    ..oracle::TAPESTRY_WARDEN
};
