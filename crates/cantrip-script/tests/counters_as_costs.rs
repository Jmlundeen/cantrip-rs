//! Counters put as a cost against the replacement effects of counters being put (CR 606.4, 118.3, 614.1a), on a table of
//! test-only cards: a replacement of counters "an effect" puts does not see a loyalty ability's cost, and one of counters "you"
//! put does. The printed cases are Doubling Season and Innkeeper's Talent with Jace, Wielder of Mysteries
//! (`scenarios/doubling-season-doubles-a-planeswalkers-entering-loyalty-but-not-its-plus-one-cost.cantrip`,
//! `scenarios/innkeepers-talent-level-three-doubles-a-planeswalkers-entering-loyalty-and-its-plus-one-cost.cantrip`).

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn nothing(_: &mut dyn Rules, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// A planeswalker of loyalty 3 with "+1: Nothing happens."
static WALKER: CardDef = CardDef {
    name: "Probe Walker",
    types: TypeSet::of(&[CardType::Planeswalker]),
    loyalty: Some(3),
    abilities: &[ability(loyalty(1), nothing, "+1: Nothing happens.")],
    ..CardDef::BLANK
};

/// "If an effect would put one or more counters on a permanent you control, it puts twice that many instead."
fn effect_puts(_: &dyn Query, asker: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::Counters(c) if c.effect && c.controller == asker.controller)
}

/// "If you would put one or more counters on a permanent, put twice that many instead."
fn you_put(_: &dyn Query, asker: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::Counters(c) if c.putter == asker.controller)
}

fn twice(_: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::Counters(c) = p {
        for n in c.counts.iter_mut() {
            *n *= 2;
        }
    }
}

static EFFECT_DOUBLER: CardDef = CardDef {
    name: "Probe Effect Doubler",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::Counters, effect_puts, twice, "twice that many")],
    ..CardDef::BLANK
};

static PUTTER_DOUBLER: CardDef = CardDef {
    name: "Probe Putter Doubler",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::Counters, you_put, twice, "twice that many")],
    ..CardDef::BLANK
};

/// An artifact planeswalker of loyalty 3 with "+1: Nothing happens." and "-1: Nothing happens."
static ARTIFACT_WALKER: CardDef = CardDef {
    name: "Probe Artifact Walker",
    types: TypeSet::of(&[CardType::Artifact, CardType::Planeswalker]),
    loyalty: Some(3),
    abilities: &[ability(loyalty(1), nothing, "+1: Nothing happens."), ability(loyalty(-1), nothing, "-1: Nothing happens.")],
    ..CardDef::BLANK
};

/// "Counters can't be put on artifacts" (as Solemnity says of artifacts, creatures, enchantments and lands).
fn no_counters_on_artifacts(q: &dyn Query, _: &Asker, object: ObjectId, _: PlayerId, _: CounterKind) -> bool {
    q.chars(object).is(CardType::Artifact)
}

static BAN: CardDef = CardDef {
    name: "Probe Ban",
    types: TypeSet::of(&[CardType::Enchantment]),
    counter_bans: &[no_counters_on_artifacts],
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&WALKER, &EFFECT_DOUBLER, &PUTTER_DOUBLER, &ARTIFACT_WALKER, &BAN, &FILLER]))
}

/// Plays Bot1's turn 1: each (kind, name, index of the cast or ability) in order, as it is offered; everything else passes.
/// The game.
fn play(first: &[&str], steps: &[(ActionKind, &str, u8)]) -> Game {
    let filler = table().find("Filler").unwrap();
    let mut deck: Vec<CardId> = first.iter().map(|n| table().find(n).unwrap()).collect();
    deck.extend(std::iter::repeat(filler).take(30 - deck.len()));
    let others = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, others], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut todo = steps.iter().peekable();
    for _ in 0..200 {
        game.run();
        if todo.peek().is_none() && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            break;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) => {
                let option = todo.peek().and_then(|&&(kind, name, index)| {
                    options.iter().find(|o| o.kind == kind && o.index == index && o.source.is_some_and(|s| game.name(s) == name))
                });
                match option {
                    Some(option) => {
                        todo.next();
                        Answer::Act(Action::of(option))
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(todo.peek().is_none(), "every step was offered");
    game
}

fn walker(game: &Game) -> ObjectId {
    *game.state.battlefield().iter().find(|&&id| game.name(id) == "Probe Walker").expect("the walker")
}

/// CR 306.5b, 614.1c: a planeswalker's entering loyalty is an effect's counters; CR 606.4, 118.3: its +1 is a cost's. A
/// replacement keyed to "an effect" doubles the first and not the second: 3 x 2 + 1 = 7.
#[test]
fn a_loyalty_cost_is_no_effect_putting_counters() {
    let game = play(
        &["Probe Effect Doubler", "Probe Walker"],
        &[(ActionKind::CastSpell, "Probe Effect Doubler", 0), (ActionKind::CastSpell, "Probe Walker", 0), (ActionKind::ActivateAbility, "Probe Walker", 0)],
    );
    assert_eq!(game.state.obj(walker(&game)).counters(CounterKind::Loyalty), 7);
}

/// CR 122.6a, 606.4: both the entering loyalty and the +1 are counters its controller puts, so a replacement keyed to who puts
/// them doubles both: 3 x 2 + 1 x 2 = 8.
#[test]
fn a_loyalty_cost_is_counters_its_controller_puts() {
    let game = play(
        &["Probe Putter Doubler", "Probe Walker"],
        &[(ActionKind::CastSpell, "Probe Putter Doubler", 0), (ActionKind::CastSpell, "Probe Walker", 0), (ActionKind::ActivateAbility, "Probe Walker", 0)],
    );
    assert_eq!(game.state.obj(walker(&game)).counters(CounterKind::Loyalty), 8);
}

/// CR 101.2, 118.3 (Solemnity's ruling): a cost that would put counters on a permanent that can't have them can't be paid, so a
/// loyalty ability with a plus is not offered while its minus is. Without the ban both are (the positive control).
#[test]
fn a_loyalty_plus_is_not_offered_where_its_counters_cant_be_put() {
    let walker_abilities = |first: &[&str], steps: &[(ActionKind, &str, u8)]| -> Vec<u8> {
        let mut game = play(first, steps);
        game.run();
        let request = game.pending().cloned().expect("Bot1's priority");
        assert_eq!(request.player, PlayerId(0));
        let RequestKind::Priority { options } = &request.kind else { panic!("a priority") };
        options
            .iter()
            .filter(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s) == "Probe Artifact Walker"))
            .map(|o| o.index)
            .collect()
    };
    assert_eq!(walker_abilities(&["Probe Artifact Walker"], &[(ActionKind::CastSpell, "Probe Artifact Walker", 0)]), vec![0, 1]);
    assert_eq!(
        walker_abilities(
            &["Probe Artifact Walker", "Probe Ban"],
            &[(ActionKind::CastSpell, "Probe Artifact Walker", 0), (ActionKind::CastSpell, "Probe Ban", 0)]
        ),
        vec![1]
    );
}
