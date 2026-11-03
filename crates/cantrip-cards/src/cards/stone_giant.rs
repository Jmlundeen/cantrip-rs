use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, targets};
use cantrip_oracle as oracle;

/// A creature its controller controls with toughness less than the Giant's power (CR 115.1, 608.2h).
fn target_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t)
        && q.chars(id).toughness.is_some_and(|toughness| toughness < q.chars(asker.source).power.unwrap_or(0)))
}

fn destroy_that_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let objects = g.event_objects(item).to_vec();
    g.destroy_all(&objects);
    Ok(())
}

fn lift(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::first_target_gains_flying(g, item)?;
    g.end_sentence();
    delayed::at_next_end_step_about(g, item, 0, &[creature]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, lift, "{T}: Target creature you control with toughness less than this creature's power gains flying until end of turn. Destroy that creature at the beginning of the next end step.")
        .targets(&[target(target_creature)])],
    delayed: &[delayed_trigger(destroy_that_creature, "Destroy that creature at the beginning of the next end step.")],
    ..oracle::STONE_GIANT
};
