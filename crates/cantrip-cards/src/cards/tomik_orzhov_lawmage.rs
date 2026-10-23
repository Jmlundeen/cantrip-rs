use cantrip_api::*;
use cantrip_mechanics::{effects, statics::{grant_delayed_ability, your_planeswalkers}, targets};
use cantrip_oracle as oracle;

/// "Planeswalkers you control have "No more than one creature can attack this planeswalker each combat."" (CR 508.1c): a
/// planeswalker its controller controls may be attacked by at most one creature in a declaration of attackers. It caps
/// the declaration alone - an effect can still put another creature onto the battlefield attacking it, or change what an
/// already-attacking creature attacks (the card's ruling). Whose planeswalkers have it is the grant's (`your_planeswalkers`),
/// so another player's planeswalker is uncapped by not having it.
/// The ability is granted to the planeswalker (`grant_delayed_ability`, layer 6 in timestamp order, CR 613.1f, 613.7), so the
/// planeswalker is capped when it has that granted ability and nothing else: one that lost all its abilities before the grant still has it,
/// one that lost them after does not. The core reads the rule only while Tomik has his abilities.
fn one_attacker(q: &dyn Query, asker: &Asker, target: Target) -> Option<u32> {
    let Target::Object(id) = target else { return None };
    let source = q.obj(asker.source);
    let granted = AbilityRef::delayed(source.copied_card().unwrap_or(source.card), 0);
    q.chars(id).abilities.contains(&Some(granted)).then_some(1)
}

/// The granted "No more than one creature can attack this planeswalker each combat": a rule of the attack declaration
/// (`one_attacker`, asked of Tomik), so it changes no characteristic of the planeswalker that has it.
fn caps_attackers(_: &dyn LayerQuery, _: ObjectId, _: &mut Chars) {}

/// "Target creature with a +1/+1 counter on it" (CR 115.1, 122.1): a creature on the battlefield with a +1/+1 counter,
/// whichever player controls it.
fn creature_with_a_plus_one_counter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).counters(CounterKind::PlusOne) > 0)
}

/// "... gains flying until end of turn." A target that has lost its counter is no longer legal and gains nothing
/// (CR 608.2b).
fn gains_flying(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Flying);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    attack_limits: &[one_attacker],
    delayed: &[granted_static(Layer::Ability, caps_attackers, "No more than one creature can attack this planeswalker each combat.")],
    statics: &[static_ability(
        Layer::Ability,
        your_planeswalkers,
        grant_delayed_ability,
        "Planeswalkers you control have \"No more than one creature can attack this planeswalker each combat.\"",
    )],
    abilities: &[ability(TAP, gains_flying, "{T}: Target creature with a +1/+1 counter on it gains flying until end of turn.")
        .targets(&[target(creature_with_a_plus_one_counter)])],
    ..oracle::TOMIK_ORZHOV_LAWMAGE
};
