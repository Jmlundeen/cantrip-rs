use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS, triggers};
use cantrip_oracle as oracle;

/// Up to one target creature can't block this turn. If you're the monarch, creatures can't block this turn: a rule about
/// every creature (CR 509.1b, 611.2c), so it reaches creatures that arrive later too.
fn cant_block(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::first_target_cant_block_this_turn(g, item)?;
    g.end_sentence();
    if g.state().player(item.controller).monarch {
        g.add_lasting_rule(item, 0, Target::None, Duration::EndOfTurn);
    }
    Ok(())
}

/// "Creatures" (CR 611.2c): each creature, asked as the rule is read.
fn creatures(q: &dyn Query, _: &Asker, _: Target, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, cantrip_mechanics::effects::become_monarch, "When Aragorn enters, you become the monarch."),
        triggered(triggers::THIS_ATTACKS, cant_block, "Whenever Aragorn attacks, up to one target creature can't block this turn. If you're the monarch, creatures can't block this turn.")
            .targets(&[targets::CREATURE.optional()]),
    ],
    lasting_rules: &[LastingRule { text: "Creatures can't block this turn.", rule: LastingRuleKind::CantBlock, applies: creatures }],
    ..oracle::ARAGORN_KING_OF_GONDOR
};
