use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell. At the beginning of your next main phase, add {C} equal to that spell's mana value. Which step
/// that is depends on when this resolves: from your own precombat main phase or your own combat it is that turn's
/// postcombat main phase, from anywhere else a precombat main phase of yours. The mana is owed whether or not the
/// spell could be countered; only an illegal target stops the whole spell (CR 608.2b).
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = cantrip_mechanics::counter_target(g, item, 0) else { return Ok(()) };
    let turn = g.state().turn;
    let step = if turn.active == item.controller
        && (Step::PrecombatMain as u8..Step::PostcombatMain as u8).contains(&(turn.step() as u8))
    {
        Step::PostcombatMain
    } else {
        Step::PrecombatMain
    };
    let this = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::Step { step, on_turn_of: Some(item.controller) },
        item.object,
        item.controller,
        AbilityRef::delayed(this, 0),
        Target::Object(spell),
    );
    Ok(())
}

/// "Add an amount of {C} equal to that spell's mana value": the spell's, as it was on the stack (CR 608.2h).
fn add_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(spell) = item.subject {
        let value = g.mana_value(spell).min(u8::MAX as u32) as u8;
        if value > 0 {
            g.add_mana(item.controller, Mana::C, value);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect: drain }),
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Add an amount of {C} equal to that spell's mana value.",
        effect: add_mana,
        ..TriggeredDef::BLANK
    })],
    ..oracle::MANA_DRAIN
};
