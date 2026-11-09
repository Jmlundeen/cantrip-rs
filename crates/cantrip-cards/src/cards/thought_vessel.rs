use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You have no maximum hand size" (CR 402.2): the cleanup step normally makes a player discard down to seven cards, and this
/// play rule removes that number for the Vessel's controller alone - the other player still discards down to seven. It is a
/// static ability of the artifact, so it applies only while the Vessel is on the battlefield under that player's control.
/// "{T}: Add {C}" is the ordinary colorless mana ability (CR 605.1a).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::C), "{T}: Add {C}.")],
    play_rules: &[PlayRule::MaximumHandSize(None)],
    ..oracle::THOUGHT_VESSEL
};
