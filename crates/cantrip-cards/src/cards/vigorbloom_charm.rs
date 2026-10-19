use cantrip_api::*;
use cantrip_mechanics::effects::{draw, gain_hexproof_and_indestructible_until_end_of_turn as protect, gain_life};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target permanent you control gains hexproof and indestructible until end of turn." Nothing happens if the target is no
/// longer legal (CR 608.2b).
fn protect_permanent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        protect(g, item.object, permanent);
    }
    Ok(())
}

/// "You draw a card and gain 3 life."
fn draw_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    draw(g, item, 1)?;
    gain_life(g, item, 3)
}

/// "Put a +1/+1 counter on target creature you control. Then it fights target creature an opponent controls." (CR 701.14,
/// 608.2b): each target is checked as it resolves. If the creature you control is no longer a legal target the counter is
/// not put and nothing fights; if only the other creature is illegal, the counter is still put and there is no fight
/// (CR 701.14b).
fn counter_then_fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    if let Some(victim) = targets::object(g, item, 1) {
        g.fight(creature, victim);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(protect_permanent, "Target permanent you control gains hexproof and indestructible until end of turn.")
        .targets(&[targets::PERMANENT_YOU_CONTROL]),
    mode(draw_and_gain, "You draw a card and gain 3 life."),
    mode(counter_then_fight, "Put a +1/+1 counter on target creature you control. Then it fights target creature an opponent controls.")
        .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::VIGORBLOOM_CHARM };
