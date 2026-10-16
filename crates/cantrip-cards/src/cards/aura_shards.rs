use cantrip_api::*;
use cantrip_mechanics::{targets, triggers, effects};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Destroy", "destroy the targeted artifact or enchantment?", "", "Destroy it", "Don't destroy");

/// You may destroy target artifact or enchantment: chosen as the ability goes on the stack, the "may" asked as it
/// resolves (CR 603.3d, 608.2d).
fn shatter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    if effects::ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        g.destroy(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::CREATURE_YOU_CONTROL_ENTERS, shatter, "Whenever a creature you control enters, you may destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    ..oracle::AURA_SHARDS
};
