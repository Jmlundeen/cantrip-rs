//! Copy effects of different durations on one permanent (CR 613.2a, 613.7, 611.2b), on a table of test-only cards. The printed
//! case is Mirrorweave's "until end of turn" under a later Thespian's Stage copy that lasts
//! (`scenarios/mirrorweave-ending-leaves-a-later-thespians-stage-copy-in-place.cantrip`).

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().battlefield().contains(&id) && q.chars(id).is(CardType::Creature))
}

fn mimic(g: &dyn Query) -> CardId {
    g.card_id(&MIMIC)
}

/// "{0}: This becomes a copy of target creature until end of turn, except it has [the second ability]."
fn copy_for_the_turn(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        let values = g.copiable_values(id).with_ability(AbilityRef { card: mimic(g), index: 1 });
        g.become_copy(item.object, item.object, values, Duration::EndOfTurn);
    }
    Ok(())
}

/// "{0}: This becomes a copy of target creature." No duration: as long as it stays on the battlefield.
fn copy_for_good(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        let values = g.copiable_values(id);
        g.become_copy(item.object, item.object, values, Duration::Indefinite);
    }
    Ok(())
}

static MIMIC: CardDef = CardDef {
    name: "Probe Mimic",
    types: TypeSet::of(&[CardType::Artifact]),
    abilities: &[
        ability(cost("{0}"), copy_for_the_turn, "{0}: This becomes a copy of target creature until end of turn, except it has the next ability.").targets(&[target(creature)]),
        ability(cost("{0}"), copy_for_good, "{0}: This becomes a copy of target creature.").targets(&[target(creature)]),
    ],
    ..CardDef::BLANK
};

static LION: CardDef = CardDef { name: "Probe Lion", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(1), ..CardDef::BLANK };
static WALL: CardDef = CardDef { name: "Probe Wall", types: TypeSet::of(&[CardType::Creature]), power: Some(0), toughness: Some(4), ..CardDef::BLANK };

/// A land to fill hands and libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&MIMIC, &LION, &WALL, &FILLER]))
}

enum Step {
    Cast(&'static str),
    /// The Mimic's nth ability (of its abilities now), at the creature of that name.
    Activate(u8, &'static str),
}

/// CR 613.2a, 613.7: the Mimic is a copy of the Lion until end of turn, then a copy of the Wall for good, a later timestamp.
/// When the first effect ends the second is still there: on the next turn it is a Wall, not a Mimic and not a Lion.
#[test]
fn a_copy_until_end_of_turn_ending_leaves_a_later_copy_that_lasts() {
    let id = |name: &str| table().find(name).unwrap();
    let mut deck = vec![id("Probe Mimic"), id("Probe Lion"), id("Probe Wall")];
    deck.extend(std::iter::repeat(id("Filler")).take(27));
    let mut game = Game::new(table(), &Setup { decks: vec![deck, vec![id("Filler"); 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut steps = vec![Step::Cast("Probe Lion"), Step::Cast("Probe Wall"), Step::Cast("Probe Mimic"), Step::Activate(0, "Probe Lion"), Step::Activate(0, "Probe Wall")].into_iter().peekable();
    let mut the_mimic = None;
    for _ in 0..300 {
        game.run();
        if steps.peek().is_none() && game.state.stack.is_empty() && game.state.turn.number == 2 {
            break;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && game.state.stack.is_empty() => {
                if the_mimic.is_none() {
                    the_mimic = game.state.battlefield().iter().copied().find(|&o| game.name(o) == "Probe Mimic");
                }
                let option = match steps.peek() {
                    Some(Step::Cast(name)) => options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == *name)),
                    Some(Step::Activate(index, _)) => options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source == the_mimic && o.index == *index),
                    None => None,
                };
                match option {
                    Some(option) => {
                        let mut action = Action::of(option);
                        if let Some(Step::Activate(_, at)) = steps.peek() {
                            let original = game.state.battlefield().iter().copied().find(|&o| Some(o) != the_mimic && game.name(o) == *at).unwrap();
                            action.targets = vec![Target::Object(original)];
                        }
                        steps.next();
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    let the_mimic = the_mimic.expect("the Mimic was cast");
    assert_eq!(game.state.turn.number, 2, "the first copy effect has ended with turn 1");
    assert_eq!(game.name(the_mimic), "Probe Wall", "the later copy effect, which lasts, is what the Mimic is a copy of");
    assert_eq!(game.chars(the_mimic).toughness, Some(4));
}
