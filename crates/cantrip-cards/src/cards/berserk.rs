use cantrip_api::*;
use cantrip_mechanics::{targets, delayed};
use cantrip_oracle as oracle;

/// "Cast this spell only before the combat damage step" (CR 601.3): any step of the turn before it.
fn before_combat_damage(q: &dyn Query, _: &Asker) -> bool {
    q.state().turn.step() < Step::CombatDamage
}

/// "Target creature gains trample and gets +X/+0 until end of turn, where X is its power. At the beginning of the next end
/// step, destroy that creature if it attacked this turn." X is the power as Berserk resolves.
fn frenzy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let power = g.chars(creature).power.unwrap_or(0);
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::Pump(power, 0), Duration::EndOfTurn);
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    });
    g.end_sentence();
    delayed::at_next_end_step(g, item, 0, Target::Object(creature));
    Ok(())
}

/// The delayed ability: destroy that creature if it attacked this turn (CR 508.1).
fn destroy_if_attacked(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && g.history().attacked_this_turn(creature)
    {
        g.destroy(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], frenzy),
    delayed: &[delayed_trigger(destroy_if_attacked, "At the beginning of the next end step, destroy that creature if it attacked this turn.")],
    cast_only_if: Some(before_combat_damage),
    ..oracle::BERSERK
};
