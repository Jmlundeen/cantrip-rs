use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{exile_and_return, targets};
use cantrip_oracle as oracle;

/// "If it's the first end step of the turn": the turn's end steps are in its history, and the one under way is the only one
/// so far. Asked as the ability resolves, after the flicker - not an intervening "if" (CR 603.4).
fn first_end_step_of_the_turn(q: &dyn Query) -> bool {
    q.history().this_turn().iter().filter(|r| matches!(r.event, Event::StepBegan { step: Step::End })).count() == 1
}

/// "Exile target creature you control, then return it to the battlefield under its owner's control. Then if it's the first
/// end step of the turn, there is an additional end step after this step." The creature comes back as a new object (CR
/// 400.7); a token exiled this way ceases to exist (CR 111.7). If the target is illegal as the ability resolves, it does
/// not resolve, and there is no additional end step (CR 608.2b; the ruling).
fn flicker_then_end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    exile_and_return(g, &[creature])?;
    if first_end_step_of_the_turn(g) {
        g.add_end_step();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        flicker_then_end_step,
        "At the beginning of your end step, exile target creature you control, then return it to the battlefield under its owner's control. Then if it's the first end step of the turn, there is an additional end step after this step.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::YSHTOLA_RHUL
};
