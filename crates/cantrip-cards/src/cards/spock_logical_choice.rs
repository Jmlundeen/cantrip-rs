use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Vigilance (Attacking doesn't cause this creature to tap.) {T}: Draw a card." (CR 702.20, 602.2a): vigilance is a core
/// keyword with no lines; the {T} ability taps Spock to draw one card, which vigilance leaves available after attacking.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, |g, item| effects::draw(g, item, 1), "{T}: Draw a card.")],
    ..oracle::SPOCK_LOGICAL_CHOICE
};
