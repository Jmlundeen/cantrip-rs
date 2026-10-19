//! Saddle N (CR 702.171a): "Tap any number of other untapped creatures you control with total power N or more: This
//! Mount becomes saddled until end of turn. Saddle only as a sorcery." Caustic Bronco prints it with 3.
//!
//! The cost is the core's (`Cost::tap_creatures`); `text` is the ability's full text, which a scenario's `Ability:`
//! matches.

use cantrip_api::*;

fn becomes_saddled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.become_saddled(item.object);
    Ok(())
}

pub const fn ability(power: u8, text: &'static str) -> AbilityDef {
    cantrip_api::ability(Cost { tap_creatures: Some(TapCreatures { power, greater_with: Some(Keyword::SaddlesAsThoughPowerTwoGreater) }), ..Cost::NONE }, becomes_saddled, text)
        .sorcery_speed()
}
