use cantrip_api::*;
use cantrip_mechanics::effects::{self, gain_indestructible_until_end_of_turn};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target artifact or creature gains indestructible until end of turn" (CR 702.12b, 611.2a): nothing if the target is no
/// longer legal as the spell resolves (CR 608.2b).
fn target_gains_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    gain_indestructible_until_end_of_turn(g, item.object, permanent);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(target_gains_indestructible, "Target artifact or creature gains indestructible until end of turn.").targets(&[targets::ARTIFACT_OR_CREATURE]),
    mode(effects::damage_first_target::<2>, "Reroute Systems deals 2 damage to target tapped creature.").targets(&[targets::TAPPED_CREATURE]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::REROUTE_SYSTEMS };
