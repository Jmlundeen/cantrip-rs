use cantrip_api::*;
use cantrip_mechanics::triggers::{ARTIFACT_YOU_CONTROL_ENTERS, YOUR_END_STEP};
use cantrip_mechanics::{artifacts_you_control, effects, targets};
use cantrip_oracle as oracle;

/// The end step ability's intervening "if" (CR 603.4): four or more artifacts, checked as the step begins and again as the
/// ability resolves.
fn four_artifacts(q: &dyn Query, asker: &Asker) -> bool {
    artifacts_you_control(q, asker.controller) >= 4
}

/// "Return target creature card from your graveyard to your hand. Then if you control eight or more artifacts, each opponent
/// loses life equal to that card's power." Nothing at all if the target is no longer legal (CR 608.2b), so no life is lost
/// either. The card's power is read where it now is, in the hand; a negative power is no life (CR 107.1b).
fn recur_and_drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let returned = g.return_to_hand(&[creature]);
    if artifacts_you_control(g, item.controller) < 8 {
        return Ok(());
    }
    let Some(&card) = returned.first() else { return Ok(()) };
    let power = u32::try_from(g.chars(card).power.unwrap_or_default()).unwrap_or_default();
    g.each_opponent_loses_life(item.controller, power as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(ARTIFACT_YOU_CONTROL_ENTERS, |g, item| effects::surveil(g, item, 1), "Whenever an artifact you control enters, surveil 1."),
        triggered(
            YOUR_END_STEP,
            recur_and_drain,
            "At the beginning of your end step, if you control four or more artifacts, return target creature card from your graveyard to your hand. Then if you control eight or more artifacts, each opponent loses life equal to that card's power.",
        )
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD])
        .condition(four_artifacts),
    ],
    ..oracle::GOLBEZ_CRYSTAL_COLLECTOR
};
