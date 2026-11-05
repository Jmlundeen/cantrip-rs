use cantrip_api::*;
use cantrip_mechanics::effects::gain_hexproof_and_indestructible_until_end_of_turn;
use cantrip_oracle as oracle;

/// "Artifacts you control gain hexproof and indestructible until end of turn." (CR 611.2, 702.11, 702.12): the effect
/// grants both to each artifact its controller controls as it resolves; an artifact that comes under their control
/// later this turn was not one of them.
fn protect_artifacts(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let artifacts: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Artifact)).collect();
    instruction(g, |g| {
        for artifact in artifacts {
            gain_hexproof_and_indestructible_until_end_of_turn(g, item.object, artifact);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], protect_artifacts),
    ..oracle::EMERGENCY_REGENERATION
};
