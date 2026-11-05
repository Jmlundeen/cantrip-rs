use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, mobilize, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +1/+0 and gains indestructible until end of turn" (CR 611.2a): both are continuous effects of the resolving
/// ability on the target, one instruction; nothing if the target has left the battlefield (CR 400.7).
fn rally(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 1, 0);
        effects::gain_indestructible_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)"),
        triggered(THIS_ENTERS, rally, "When this creature enters, target creature gets +1/+0 and gains indestructible until end of turn.").targets(&[targets::CREATURE]),
    ],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::REIGNING_VICTOR
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
