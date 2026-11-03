use cantrip_api::*;
use cantrip_mechanics::{combat, delayed};
use cantrip_oracle as oracle;

/// "Creatures the active player controls attack this turn if able" (CR 508.1d): every creature of the player whose turn it is.
fn lasting_rule_0(q: &dyn Query, _: &Asker, _: Target, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Creature) && q.state().turn.active == q.controller(id)
}

/// "At the beginning of the next end step, destroy all non-Wall creatures that player controls that didn't attack this turn.
/// Ignore this effect for each creature the player didn't control continuously since the beginning of the turn." (CR 603.7,
/// 701.8a, 302.6).
fn delayed_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let objects_3 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                !(q.chars(id).has_subtype(q.words(item.object).subtype(Subtype::Wall)))
                    && q.chars(id).is(CardType::Creature)
                    && item.subject.player() == Some(q.controller(id))
                    && !q.attacked_this_turn(id)
                    && q.state().obj(id).controlled_since < q.state().turn.number
            })
            .collect::<Vec<ObjectId>>()
    };
    g.destroy_all(&objects_3);
    Ok(())
}

/// "Creatures the active player controls attack this turn if able. At the beginning of the next end step, destroy all non-Wall
/// creatures that player controls that didn't attack this turn." (CR 508.1d, 603.7): the delayed ability is about the active
/// player.
fn cast_spell_only_during_opponents(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    g.add_lasting_rule(item, 0, Target::None, Duration::EndOfTurn);
    g.end_sentence();
    let active = g.state().turn.active;
    delayed::at_next_end_step(g, item, 0, Target::Player(active));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], cast_spell_only_during_opponents),
    delayed: &[
        delayed_trigger(
            delayed_2,
            "At the beginning of the next end step, destroy all non-Wall creatures that player controls that didn't attack this turn. Ignore this effect for each creature the player didn't control continuously since the beginning of the turn.",
        ),
    ],
    cast_only_if: Some(combat::during_opponents_turn_before_attackers),
    lasting_rules: &[
        LastingRule {
            text: "Creatures the active player controls attack this turn if able.",
            rule: LastingRuleKind::MustAttack,
            applies: lasting_rule_0,
        },
    ],
    ..oracle::SIRENS_CALL
};
