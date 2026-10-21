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

/// Target creature gets -1/-1 until end of turn, then +1/+1 until end of turn: two effects, no event between or after them.
fn flicker(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.targets()[0] {
        cantrip_mechanics::pump(g, item.object, id, -1, -1);
        cantrip_mechanics::pump(g, item.object, id, 1, 1);
    }
    Ok(())
}

/// "{T}: Target creature gets -1/-1 until end of turn, then gets +1/+1 until end of turn." Its toughness is 0 only between
/// the two effects, in the middle of the resolution.
static FLICKER: CardDef = CardDef {
    name: "Probe Flickerer",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, flicker, "{T}: Target creature gets -1/-1 until end of turn, then gets +1/+1 until end of turn.")
        .targets(&[target(creature)])],
    ..CardDef::BLANK
};

/// "When you control no artifacts, you gain 3 life."
fn no_artifacts(q: &dyn Query, asker: &Asker) -> bool {
    !q.state().battlefield().iter().any(|&id| q.controller(id) == asker.controller && q.chars(id).is(CardType::Artifact))
}

static HERMIT: CardDef = CardDef {
    name: "Probe Hermit",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[state_triggered(no_artifacts, gain_three, "When you control no artifacts, you gain 3 life.")],
    ..CardDef::BLANK
};

fn opponents_artifact(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().battlefield().contains(&id) && q.controller(id) != asker.controller && q.chars(id).is(CardType::Artifact))
}

/// Exchange control of this artifact and target artifact an opponent controls.
fn trade(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(theirs) = item.targets()[0] {
        g.exchange_control(item.object, item.object, theirs);
    }
    Ok(())
}

/// "{T}: Exchange control of this artifact and target artifact an opponent controls." One instruction (CR 701.12a), though
/// the engine gives each permanent in turn.
static TRADER: CardDef = CardDef {
    name: "Probe Trader",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, trade, "{T}: Exchange control of this artifact and target artifact an opponent controls.")
        .targets(&[target(opponents_artifact)])],
    ..CardDef::BLANK
};

/// An artifact that does nothing.
static BAUBLE: CardDef = CardDef { name: "Probe Bauble", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

/// A land to fill hands and libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&FRAIL, &SHRINK, &FLICKER, &HERMIT, &TRADER, &BAUBLE, &FILLER]))
}

/// CR 603.8, 704.3: the -1/-1 ability resolves and the Frail is a 0/0 with no event to ask its state after; its state trigger has
/// triggered all the same, before the state-based action puts it into the graveyard, and resolves with its source gone.
#[test]
fn a_state_true_after_a_resolution_triggers_though_a_state_based_action_ends_it() {
    let game = play("Probe Shrinker");
    assert!(game.state.battlefield().iter().all(|&id| game.name(id) != "Probe Frail"), "the 0/0 is put into the graveyard");
    assert_eq!(game.state.player(PlayerId(0)).life, 23, "the state trigger triggered before the state-based action and resolved");
}

/// CR 603.8: "triggers as soon as the game state matches the condition" - the Frail's toughness is 0 between the two effects
/// of one resolution, with no event before the second puts it back to 1; the state trigger triggers there all the same, and
/// resolves though its state is false again. The Frail survives.
#[test]
fn a_state_true_only_in_the_middle_of_a_resolution_triggers() {
    let game = play("Probe Flickerer");
    assert!(game.state.battlefield().iter().any(|&id| game.name(id) == "Probe Frail"), "the Frail is a 1/1 again");
    assert_eq!(game.state.player(PlayerId(0)).life, 23, "the state trigger triggered mid-resolution and resolved");
}

/// The first seat casts the Probe Frail and `artifact`, then activates the artifact at the Frail, and everything resolves.
fn play(artifact: &'static str) -> Game {
    let frail = table().find("Probe Frail").unwrap();
    let shrink = table().find(artifact).unwrap();
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![frail, shrink];
    deck.extend(std::iter::repeat(filler).take(28));
    let others = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, others], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut cast = [(ActionKind::CastSpell, "Probe Frail"), (ActionKind::CastSpell, artifact), (ActionKind::ActivateAbility, artifact)].into_iter().peekable();
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
    game
}

/// CR 701.12a, 603.8: an exchange of control is one instruction. Bot1's only artifact, the Trader, is exchanged for Bot2's
/// Bauble; Bot1 controls an artifact before and after, and its "when you control no artifacts" is not triggered by the
/// moment between the engine giving the Trader and taking the Bauble.
#[test]
fn an_exchange_of_control_is_not_judged_half_done() {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    for (name, seat) in [("Probe Trader", 0), ("Probe Hermit", 0), ("Probe Bauble", 1)] {
        let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), PlayerId(seat), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
    }
    game.verify = true;
    let mut traded = false;
    for _ in 0..100 {
        game.run();
        if traded && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            break;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && !traded => {
                match options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s) == "Probe Trader")) {
                    Some(option) => {
                        traded = true;
                        let mut action = Action::of(option);
                        action.targets = vec![option.target_candidates[0]];
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
    let bauble = game.state.battlefield().iter().copied().find(|&id| game.name(id) == "Probe Bauble").expect("the Bauble");
    assert_eq!(game.state.obj(bauble).controller, PlayerId(0), "the exchange happened");
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "no moment without an artifact was judged");
}
