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

/// "Target creature gets -1/-1 until end of turn. Then it gets +1/+1 until end of turn.": two sentences (`Game::end_sentence`
/// between them), two effects, no event between or after them.
fn flicker(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_mechanics::pump(g, item.object, id, -1, -1);
        g.end_sentence();
        cantrip_mechanics::pump(g, item.object, id, 1, 1);
    }
    Ok(())
}

/// "{T}: Target creature gets -1/-1 until end of turn. Then it gets +1/+1 until end of turn." Its toughness is 0 only between
/// the two effects, in the middle of the resolution.
static FLICKER: CardDef = CardDef {
    name: "Probe Flickerer",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, flicker, "{T}: Target creature gets -1/-1 until end of turn. Then it gets +1/+1 until end of turn.")
        .targets(&[target(creature)])],
    ..CardDef::BLANK
};

/// "Target creature gets -1/-1 and +1/+1 until end of turn": one sentence, two effects - no point of the game between them.
fn evened(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_api::instruction(g, |g| {
            cantrip_mechanics::pump(g, item.object, id, -1, -1);
            cantrip_mechanics::pump(g, item.object, id, 1, 1);
        });
    }
    Ok(())
}

/// "{T}: Target creature gets -1/-1 and +1/+1 until end of turn." One instruction: the creature's toughness is never 0 at a
/// point of the game (CR 608.2c, 611.2c).
static EVENER: CardDef = CardDef {
    name: "Probe Evener",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, evened, "{T}: Target creature gets -1/-1 and +1/+1 until end of turn.").targets(&[target(creature)])],
    ..CardDef::BLANK
};

/// "Target creature gets -1/-1 until end of turn. Then it gets +1/+1 until end of turn." written as `flicker` is, but with
/// nothing said between the two effects: neither that they are one instruction nor that one ends between them.
fn unsaid(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_mechanics::pump(g, item.object, id, -1, -1);
        cantrip_mechanics::pump(g, item.object, id, 1, 1);
    }
    Ok(())
}

static UNSAID: CardDef = CardDef {
    name: "Probe Unsaid",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, unsaid, "{T}: Target creature gets -1/-1 until end of turn. Then it gets +1/+1 until end of turn.").targets(&[target(creature)])],
    ..CardDef::BLANK
};

fn minus_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_mechanics::pump(g, item.object, id, -1, -1);
    }
    Ok(())
}

fn plus_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        cantrip_mechanics::pump(g, item.object, id, 1, 1);
    }
    Ok(())
}

static CHARM_MODES: ModalDef = modes(&[
    mode(minus_one, "Target creature gets -1/-1 until end of turn.").targets(&[target(creature)]),
    mode(plus_one, "Target creature gets +1/+1 until end of turn.").targets(&[target(creature)]),
])
.choose(2, 2);

/// "Choose both - Target creature gets -1/-1 until end of turn. - Target creature gets +1/+1 until end of turn." No
/// `end_sentence` in it: the modes are followed in the order printed (CR 700.2d, 608.2c), the second after the first is done.
static CHARM: CardDef = CardDef {
    name: "Probe Charm",
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    modal: Some(&CHARM_MODES),
    ..CardDef::BLANK
};

/// Stage 0: "Target creature gets -1/-1 until end of turn", then a question - you choose a card in your hand - that stops the
/// run (`Game::ask_then`); stage 1 changes nothing more. The effect has begun before the question and is still there after.
fn shrink_then_ask(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    match g.stage() {
        0 => {
            if let Target::Object(id) = item.target(0) {
                cantrip_mechanics::pump(g, item.object, id, -1, -1);
            }
            let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
            let eligible = hand.iter().copied().take(1).collect();
            let kind = RequestKind::choose_cards(item.object, Purpose::Reveal, hand, eligible, 1, false, "");
            Err(g.ask_then(1, item.controller, kind))
        }
        _ => Ok(()),
    }
}

/// "{T}: Target creature gets -1/-1 until end of turn. Choose a card in your hand." written as explicit stages.
static ASKER: CardDef = CardDef {
    name: "Probe Asker",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[ability(TAP, shrink_then_ask, "{T}: Target creature gets -1/-1 until end of turn. Choose a card in your hand.").targets(&[target(creature)])],
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
    if let Target::Object(theirs) = item.target(0) {
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
    TABLE.get_or_init(|| CardTable::new([&FRAIL, &SHRINK, &FLICKER, &EVENER, &UNSAID, &CHARM, &ASKER, &HERMIT, &TRADER, &BAUBLE, &FILLER]))
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

/// Two effects the card says are one instruction (`cantrip_api::instruction`): the Frail's toughness is never 0 where the
/// game could see it, and its state trigger does not trigger.
#[test]
fn a_state_true_only_inside_one_instruction_does_not_trigger() {
    let game = play("Probe Evener");
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "no point of the game had the Frail at 0 toughness");
}

/// CR 603.8, 608.2c: a card that changes the game with no event twice and says neither that the changes are one
/// instruction nor that one ends between them is refused under verification, naming it - the engine can't know which its
/// text means, and the corpus, which runs every line of every card under `CANTRIP_VERIFY`, finds it.
#[test]
fn a_card_that_says_neither_where_an_instruction_ends_nor_that_it_is_one_is_refused() {
    let Err(refused) = std::panic::catch_unwind(|| play("Probe Unsaid")) else { panic!("the resolution is not refused") };
    let message = refused.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(message.contains("Probe Unsaid's resolution changed the game twice"), "{message}");
}

/// CR 603.8, 700.2d, 608.2c: a modal spell's modes are followed in order, each done before the next - with no
/// `end_sentence` in the card, the Frail is 0/0 between the -1/-1 mode and the +1/+1 mode, and its state trigger triggers
/// there.
#[test]
fn a_state_true_between_two_modes_triggers() {
    let game = play_steps(&[(ActionKind::CastSpell, "Probe Frail"), (ActionKind::CastSpell, "Probe Charm")], "Probe Charm");
    let charm = game.state.zone(Zone::graveyard(PlayerId(0))).iter().any(|&id| game.name(id) == "Probe Charm");
    assert!(charm, "the Charm resolved");
    assert!(game.state.battlefield().iter().any(|&id| game.name(id) == "Probe Frail"), "the Frail is a 1/1 again");
    assert_eq!(game.state.player(PlayerId(0)).life, 23, "the state trigger triggered between the modes and resolved");
}

/// CR 603.8, 704.3: a staged resolution that stops at a question (`Game::ask_then`) keeps what it did before it, and what
/// that changed with no event is judged as the resolution goes on - here once it ends, before the 0-toughness state-based
/// action puts the Frail into the graveyard.
#[test]
fn a_change_before_a_staged_question_is_judged_when_the_resolution_goes_on() {
    let game = play("Probe Asker");
    assert!(game.state.battlefield().iter().all(|&id| game.name(id) != "Probe Frail"), "the 0/0 is put into the graveyard");
    assert_eq!(game.state.player(PlayerId(0)).life, 23, "the state trigger triggered before the state-based action and resolved");
}

/// The first seat casts the Probe Frail and `artifact`, then activates the artifact at the Frail, and everything resolves.
fn play(artifact: &'static str) -> Game {
    play_steps(&[(ActionKind::CastSpell, "Probe Frail"), (ActionKind::CastSpell, artifact), (ActionKind::ActivateAbility, artifact)], artifact)
}

/// The first seat takes `steps` in order, aiming each at the Probe Frail, and everything resolves; `card` is the second
/// card of its deck.
fn play_steps(steps: &[(ActionKind, &'static str)], card: &'static str) -> Game {
    let frail = table().find("Probe Frail").unwrap();
    let shrink = table().find(card).unwrap();
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![frail, shrink];
    deck.extend(std::iter::repeat(filler).take(28));
    let others = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, others], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let frail_target = |game: &Game, candidates: &[Target]| {
        candidates.iter().copied().find(|t| matches!(t, Target::Object(id) if game.name(*id) == "Probe Frail")).unwrap_or(candidates[0])
    };
    let mut cast = steps.iter().copied().peekable();
    for _ in 0..100 {
        game.run();
        let at_priority = matches!(game.pending().map(|r| &r.kind), Some(RequestKind::Priority { .. }));
        if cast.peek().is_none() && at_priority && game.state.stack.is_empty() && game.state.triggers.is_empty() {
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
                            action.targets = vec![frail_target(&game, &option.target_candidates)];
                        }
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::ChooseModes { eligible, count, .. } => Answer::Modes(eligible.iter().copied().take(*count as usize).collect()),
            RequestKind::ChooseTarget { candidates, .. } => Answer::Targets(vec![frail_target(&game, candidates)]),
            RequestKind::ChooseCards { .. } => Answer::Cards(first_choices(&game, &request)),
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
