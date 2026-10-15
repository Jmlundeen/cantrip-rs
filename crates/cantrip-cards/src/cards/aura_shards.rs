use cantrip_api::*;
use cantrip_mechanics::{targets, triggers, effects};
use cantrip_oracle as oracle;

/// You may destroy target artifact or enchantment: chosen as the ability goes on the stack, the "may" asked as it
/// resolves (CR 603.3d, 608.2d).
fn shatter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    if effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.destroy(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature you control enters, you may destroy target artifact or enchantment.",
        listens: EventMask::ZONE_CHANGE,
        matches: triggers::creature_you_control_enters,
        targets: &[targets::ARTIFACT_OR_ENCHANTMENT],
        effect: shatter,
        ..TriggeredDef::BLANK
    })],
    ..oracle::AURA_SHARDS
};
