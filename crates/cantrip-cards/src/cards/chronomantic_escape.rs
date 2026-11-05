use cantrip_api::*;
use cantrip_mechanics::suspend;
use cantrip_oracle as oracle;

/// \"Until your next turn, creatures can't attack you. Exile Chronomantic Escape with three time counters on it.\" The rule is
/// about no object (CR 611.2c): it takes in each creature as the rule is read, whoever controls it then, and lasts until the
/// turn of the player it was about begins (CR 611.2b). The card exiles itself as it resolves, whatever it is doing there,
/// with the three time counters suspend would have put on it (CR 702.62a).
fn forbid_attacks(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.add_lasting_rule(item, 0, Target::Player(you), Duration::UntilNextTurnOf(you));
    g.end_sentence();
    suspend::exile_with_time_counters(g, item.object, 3);
    Ok(())
}

/// \"Creatures\": every creature, whoever controls it, as the rule is read (CR 611.2c).
fn every_creature(q: &dyn Query, _: &Asker, _: Target, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], forbid_attacks),
    abilities: &[suspend::ability(mana("{2}{W}"), "Suspend 3\u{2014}{2}{W}")],
    suspend: Some(SuspendDef { count: 3, cost: mana("{2}{W}") }),
    lasting_rules: &[LastingRule {
        text: "Until your next turn, creatures can't attack you.",
        rule: LastingRuleKind::CantAttackYou,
        applies: every_creature,
    }],
    ..oracle::CHRONOMANTIC_ESCAPE
};
