//! Goad (CR 701.15) and "attacks that player this combat if able" (CR 508.1d) are effects, not abilities: a creature under
//! one that then loses all its abilities (CR 613.1f, a later timestamp) still attacks each combat if able. A gained "this
//! creature attacks each combat if able" (`EffectKind::MustAttack`, Carnage's) is an ability, and is lost. Over test cards.

use crate::{Game, Setup};
use cantrip_api::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Attack Effect Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };
static BEAR: CardDef =
    CardDef { name: "Attack Effect Test Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    Box::leak(Box::new(CardTable::new([&FILLER, &BEAR])))
}

/// P0's Bear under `kind` from P1, then losing all its abilities: the most requirements P0's declaration must obey.
fn required_after_losing_abilities(kind: EffectKind) -> u32 {
    let table = table();
    let deck = vec![table.id_of(&FILLER); 30];
    let mut g = Game::new(table, &Setup { decks: vec![deck.clone(), deck], ..Default::default() });
    g.verify = true;
    let bear = g.state.push_obj(Obj::new(table.id_of(&BEAR), P0, Zone::BATTLEFIELD));
    g.state.zone_push(Zone::BATTLEFIELD, bear);
    for (controller, kind) in [(P1, kind), (P1, EffectKind::LoseAllAbilities)] {
        let timestamp = g.state.timestamp();
        g.state.effect_push(ContinuousEffect { source: bear, controller, timestamp, affected: bear, kind, duration: Duration::EndOfTurn });
    }
    for _ in 0..200 {
        g.run();
        let request = g.pending().cloned().expect("a question");
        if let RequestKind::DeclareAttackers { requirements, .. } = &request.kind {
            return requirements.most;
        }
        let answer = match &request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        g.answer(request.id, request.player, answer).unwrap();
    }
    panic!("no declaration of attackers");
}

#[test]
fn a_goaded_creature_that_loses_its_abilities_is_still_goaded() {
    // A game of two: attacking the goader, the only opponent, obeys "attack"; "a player other than the goader" can't be obeyed.
    assert_eq!(required_after_losing_abilities(EffectKind::Goaded), 1);
}

#[test]
fn a_creature_that_must_attack_a_player_and_loses_its_abilities_still_must() {
    assert_eq!(required_after_losing_abilities(EffectKind::MustAttackPlayer(P1)), 1);
}

#[test]
fn a_gained_attacks_each_combat_ability_is_lost_with_the_others() {
    assert_eq!(required_after_losing_abilities(EffectKind::MustAttack), 0);
}
