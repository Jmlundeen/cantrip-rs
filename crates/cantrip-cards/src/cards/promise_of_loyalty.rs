use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::select;
use cantrip_oracle as oracle;

/// "Each player puts a vow counter on a creature they control and sacrifices the rest." The players choose in turn order
/// (CR 101.4, 800.4f), each keeping one creature - no choice where they control one or none - and then the rule is made
/// (CR 611.2c) about the player the spell's "you" is, taking in each creature as the rule is read while it has a vow counter.
fn vow_and_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let first = g.state().turn.active;
    for player in g.state().turn_order_from(first).collect::<Vec<_>>() {
        let mut creatures: Vec<ObjectId> = g
            .state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| g.state().obj(id).controller == player && g.chars(id).is(CardType::Creature))
            .collect();
        if creatures.is_empty() {
            continue;
        }
        let kept = if creatures.len() == 1 {
            creatures[0]
        } else {
            let rule = select::Selection::new(select::exactly(1)).called("creature");
            match choose::select(g, player, item.object, Purpose::Keep, creatures.clone(), rule)?.first() {
                Some(&id) => id,
                None => continue,
            }
        };
        g.end_sentence();
        creatures.retain(|&id| id != kept);
        g.add_counters(kept, CounterKind::Vow, 1);
        if !creatures.is_empty() {
            g.sacrifice(player, &creatures);
        }
    }
    g.add_lasting_rule(item, 0, Target::Player(item.controller), Duration::Indefinite);
    Ok(())
}

/// "Each of those creatures" (CR 611.2b): the creatures that still have a vow counter on them, as the rule is read - the
/// promise is the creature's own, so it holds whoever controls it and ends the moment the last vow counter leaves it.
fn has_a_vow_counter(q: &dyn Query, _: &Asker, _: Target, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature) && q.state().obj(id).counters(CounterKind::Vow) > 0
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], vow_and_sacrifice),
    lasting_rules: &[LastingRule {
        text: "That creature can't attack you or planeswalkers you control for as long as it has a vow counter on it.",
        rule: LastingRuleKind::CantAttackYouOrYourPlaneswalkers,
        applies: has_a_vow_counter,
    }],
    ..oracle::PROMISE_OF_LOYALTY
};
