use cantrip_api::*;
use cantrip_mechanics::{fight, pump, targets};
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if you control a permanent with mana value 4 or greater" (CR 601.2f, 118.7a): judged once, as
/// the total cost is determined, over every permanent the caster controls, lands and tokens included (a token has mana value 0,
/// and a permanent with {X} in its cost counts X as 0, CR 202.3e); losing the permanent afterwards changes nothing paid.
fn less_with_a_big_permanent(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.permanents_of(player).into_iter().any(|id| q.mana_value(id) >= 4) { CostDelta::less(2) } else { CostDelta::NONE }
}

/// "One or two target creatures you control each get +1/+0 until end of turn. They each deal damage equal to their power to target
/// creature an opponent controls." (CR 608.2b, 608.2c): the pumps are one instruction done first, so the damage is the power they
/// have now, and they are done whether or not the opponent's creature is still a legal target; with it illegal nothing is dealt.
fn team_up(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let sources = item.target_count as usize - 1;
    g.one_instruction(&mut |g| {
        for slot in 0..sources {
            if let Some(creature) = targets::object(g, item, slot) {
                pump(g, item.object, creature, 1, 0);
            }
        }
        Ok(())
    })?;
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    fight::targets_each_bite_last(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL.between(1, 2), targets::CREATURE_AN_OPPONENT_CONTROLS], team_up),
    own_cost_changes: &[less_with_a_big_permanent],
    ..oracle::TERRIFIC_TEAM_UP
};
