//! State triggers (CR 603.8) against the state-based actions that follow a resolution (CR 704.3), on a table of test-only
//! cards. The printed case is Thespian's Stage becoming a Dark Depths with no ice counters and losing the legend rule
//! (`scenarios/thespians-stage-copying-dark-depths-keeping-the-original-makes-no-marit-lage.cantrip`); this one has no
//! copy effect and no legend in it: a continuous effect makes the state true with no event recorded, and a state-based action
//! ends it before anyone would receive priority.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

/// "When this creature's toughness is 0 or less, you gain 3 life": a state trigger whose state the 0-toughness state-based
/// action (CR 704.5f) ends at once.
fn toughness_gone(q: &dyn Query, asker: &Asker) -> bool {
    q.chars(asker.source).toughness.is_some_and(|t| t <= 0)
}

fn gain_three(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 3);
    Ok(())
}

static FRAIL: CardDef = CardDef {
    name: "Probe Frail",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[state_triggered(toughness_gone, gain_three, "When this creature's toughness is 0 or less, you gain 3 life.")],
    ..CardDef::BLANK
};

fn shrink(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_mechanics::pump(g, item.object, id, -1, -1);
    }
    Ok(())
}

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().battlefield().contains(&id) && q.chars(id).is(CardType::Creature))
}

/// A zero-cost artifact: "{T}: Target creature gets -1/-1 until end of turn." The ability's resolution moves nothing and the
/// continuous effect it makes records no event, so no event asks the state after it (an instant would, going to the
/// graveyard).
static SHRINK: CardDef = CardDef {
    name: "Probe Shrinker",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, shrink, "{T}: Target creature gets -1/-1 until end of turn.").targets(&[target(creature)])],
    ..CardDef::BLANK
};

/// A land to fill hands and libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&FRAIL, &SHRINK, &FILLER]))
}

/// CR 603.8, 704.3: the -1/-1 ability resolves and the Frail is a 0/0 with no event to ask its state after; its state trigger has
/// triggered all the same, before the state-based action puts it into the graveyard, and resolves with its source gone.
#[test]
fn a_state_true_after_a_resolution_triggers_though_a_state_based_action_ends_it() {
    let frail = table().find("Probe Frail").unwrap();
    let shrink = table().find("Probe Shrinker").unwrap();
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![frail, shrink];
    deck.extend(std::iter::repeat(filler).take(28));
    let others = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, others], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut cast = [(ActionKind::CastSpell, "Probe Frail"), (ActionKind::CastSpell, "Probe Shrinker"), (ActionKind::ActivateAbility, "Probe Shrinker")].into_iter().peekable();
    for _ in 0..100 {
        game.run();
        if cast.peek().is_none() && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            break;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) => {
                let option = cast.peek().and_then(|&(kind, name)| options.iter().find(|o| o.kind == kind && o.source.is_some_and(|s| game.name(s) == name)));
                match option {
                    Some(option) => {
                        cast.next();
                        let mut action = Action::of(option);
                        if option.targets_required > 0 {
                            action.targets = vec![option.target_candidates[0]];
                        }
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(game.state.battlefield().iter().all(|&id| game.name(id) != "Probe Frail"), "the 0/0 is put into the graveyard");
    assert_eq!(game.state.player(PlayerId(0)).life, 23, "the state trigger triggered before the state-based action and resolved");
}
