//! Firebending N (CR 702.189a): "Whenever this creature attacks, add N {R}. This mana lasts until end of combat."
//! Avatar Roku and the Dragon token his Saga's back face makes print it with 4.
//!
//! A card with firebending lists `trigger::<N>(text)` among its `abilities`.

use cantrip_api::*;

fn attacks(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Attacks { attacker, .. } if *attacker == source => Fired::PLAIN,
        _ => None,
    }
}

fn firebend<const N: u8>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana_until_end_of_combat(item.controller, Mana::R, N);
    Ok(())
}

/// `text` is the card's own reminder text.
pub const fn trigger<const N: u8>(text: &'static str) -> AbilityDef {
    triggered(when(EventMask::ATTACKS, attacks), firebend::<N>, text)
}
