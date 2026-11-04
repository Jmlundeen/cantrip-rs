use cantrip_api::*;
use cantrip_mechanics::{delayed, targets, triggers};
use cantrip_oracle as oracle;

/// "Choose target creature you control and target Equipment. Gain control of that Equipment until end of turn. Attach it to
/// the chosen creature. When you lose control of that Equipment this turn, if it's attached to a creature you control,
/// unattach it." The control is gained whether or not the creature is still a legal target (CR 608.2b; its ruling: "you'll
/// still gain control of the target Equipment until end of turn, but you won't attach it"). The delayed ability (CR 603.7b)
/// watches the Equipment.
fn stolen_uniform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(equipment) = targets::object(g, item, 1) else { return Ok(()) };
    g.gain_control(item.object, equipment, item.controller, Duration::EndOfTurn);
    g.end_sentence();
    if let Some(creature) = targets::object(g, item, 0) {
        g.attach(equipment, creature);
    }
    delayed::this_turn(g, item, equipment);
    Ok(())
}

/// "If it's attached to a creature you control" (CR 603.4): asked as it triggers and again as it resolves.
fn attached_to_your_creature(q: &dyn Query, me: &Asker) -> bool {
    q.state()
        .obj(me.source)
        .attached_to
        .is_some_and(|host| q.is_on_battlefield(host) && q.controller(host) == me.controller && q.chars(host).is(CardType::Creature))
}

/// "Unattach it" (CR 701.3d).
fn unattach_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.unattach(item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::EQUIPMENT], stolen_uniform),
    delayed: &[triggered(
        triggers::YOU_LOSE_CONTROL_OF_THIS,
        unattach_it,
        "When you lose control of that Equipment this turn, if it's attached to a creature you control, unattach it.",
    )
    .condition(attached_to_your_creature)],
    ..oracle::STOLEN_UNIFORM
};
