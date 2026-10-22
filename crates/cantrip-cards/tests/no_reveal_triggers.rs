//! A card revealed because its revealing triggered an ability stays revealed until that ability leaves the stack (CR
//! 701.20a), which the look-back rule asks (CR 603.10a, `ops::seen_by_all`). The engine knows which ability a reveal
//! triggered only for miracle (CR 702.94a), whose triggered ability names the card it revealed: no other triggered ability
//! may listen for a reveal (`Event::Revealed`) until the engine records what such an ability was triggered by.

use cantrip_api::*;

#[test]
fn no_triggered_ability_listens_for_a_reveal() {
    let table = CardTable::new(cantrip_cards::all());
    let listening: Vec<&str> = table
        .cards
        .iter()
        .filter(|c| {
            c.abilities.iter().chain(c.delayed).any(|a| matches!(a, AbilityDef::Triggered(t) if t.listens.intersects(EventMask::REVEALED)))
        })
        .map(|c| c.name)
        .collect();
    assert!(listening.is_empty(), "triggered abilities that listen for a reveal, which `ops::seen_by_all` can't tell apart: {listening:?}");
}
