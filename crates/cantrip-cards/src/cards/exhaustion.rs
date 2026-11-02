use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Creatures and lands [target opponent] controls" (CR 611.2c): asked as the rule is read, so it takes in those that
/// arrive under that player's control later.
fn creatures_and_lands_they_control(q: &dyn Query, _: &Asker, about: Target, id: ObjectId) -> bool {
    (q.chars(id).is(CardType::Creature) || q.chars(id).is(CardType::Land)) && about.player() == Some(q.controller(id))
}

/// "Creatures and lands target opponent controls don't untap during their next untap step." (CR 502.3, 611.2c): a rule
/// about that player's permanents, in that player's next untap step alone (the Exhaustion ruling).
fn dont_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(p) = g.state().target(item, 0) else { return Ok(()) };
    g.add_lasting_rule(item, 0, Target::Player(p), Duration::DuringNextUntapStepOf(p));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], dont_untap),
    lasting_rules: &[LastingRule {
        text: "Creatures and lands target opponent controls don't untap during their next untap step.",
        rule: LastingRuleKind::DoesntUntap,
        applies: creatures_and_lands_they_control,
    }],
    ..oracle::EXHAUSTION
};
