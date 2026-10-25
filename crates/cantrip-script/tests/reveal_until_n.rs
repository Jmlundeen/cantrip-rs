//! "Reveal cards from the top of your library until you reveal N creature cards" (`cantrip_mechanics::library::reveal_until_n`,
//! CR 701.20a), on a table of test-only cards: the reveal runs from the top down to the Nth creature card, or through the
//! whole library when it holds fewer, and reveals nothing for N of 0. The printed case is Dack Fayden, Helping Hand (X the
//! number of opponents you have); test cards stand in until it is ported. Each probe puts the creature cards found into its
//! caster's hand and the other revealed cards into the graveyard, so where each card ends shows what was revealed.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::library::reveal_until_n;

fn reveal(g: &mut dyn Rules, item: &StackItem, n: usize) -> Result<(), Suspend> {
    let (found, rest) = reveal_until_n(g, item.controller, n, &|q, c| q.chars(c).is(CardType::Creature));
    g.return_to_hand_revealed(&found);
    g.put_into_graveyard(&rest);
    Ok(())
}

static REVEAL_TWO: CardDef = CardDef {
    name: "Probe Reveal Two",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], |g, item| reveal(g, item, 2)),
    ..CardDef::BLANK
};
static REVEAL_FIVE: CardDef = CardDef {
    name: "Probe Reveal Five",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], |g, item| reveal(g, item, 5)),
    ..CardDef::BLANK
};
static REVEAL_NONE: CardDef = CardDef {
    name: "Probe Reveal None",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], |g, item| reveal(g, item, 0)),
    ..CardDef::BLANK
};

static BEAR: CardDef =
    CardDef { name: "Probe Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&REVEAL_TWO, &REVEAL_FIVE, &REVEAL_NONE, &BEAR, &FILLER]))
}

/// What became of Bot1's library after it cast `probe`.
struct Outcome {
    bears_in_hand: usize,
    graveyard: usize,
    library: usize,
    revealed: usize,
}

/// Bot1 holds `probe` and six Fillers; its library, top first, is Filler, Bear, Filler, Filler, Bear, Bear, then `tail`
/// Fillers. It casts `probe`.
fn cast(probe: &str, tail: usize) -> Outcome {
    let find = |n: &str| table().find(n).unwrap();
    let mut first = vec![find(probe)];
    first.extend(std::iter::repeat_n(find("Filler"), 6));
    for n in ["Filler", "Probe Bear", "Filler", "Filler", "Probe Bear", "Probe Bear"] {
        first.push(find(n));
    }
    first.extend(std::iter::repeat_n(find("Filler"), tail));
    let second = vec![find("Filler"); 30];
    let mut game = Game::new(table(), &Setup { decks: vec![first, second], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let bot1 = PlayerId(0);
    let mut cast = false;
    for _ in 0..200 {
        game.run();
        if cast && game.state.stack.is_empty() {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == bot1 && !cast && game.state.stack.is_empty() => {
                match options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == probe)) {
                    Some(option) => {
                        cast = true;
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
    assert!(cast && game.state.stack.is_empty(), "the probe resolved");
    let bears_in_hand = game.state.zone(Zone::hand(bot1)).iter().filter(|&&c| game.name(c) == "Probe Bear").count();
    Outcome {
        bears_in_hand,
        graveyard: game.state.zone(Zone::graveyard(bot1)).len(),
        library: game.state.zone(Zone::library(bot1)).len(),
        revealed: game.history().count(EventQuery::kind(EventMask::REVEALED)),
    }
}

/// Two creature cards: the reveal stops at the second Bear, the fifth card - the three Fillers above it are the rest, and
/// the third Bear and everything under it stay unrevealed.
#[test]
fn the_reveal_stops_at_the_nth_qualifying_card() {
    let o = cast("Probe Reveal Two", 10);
    assert_eq!((o.bears_in_hand, o.graveyard, o.library, o.revealed), (2, 3 + 1, 11, 5));
}

/// Five creature cards from a library holding three: the whole library is revealed, the three Bears are found and every
/// Filler is the rest.
#[test]
fn a_library_holding_fewer_is_revealed_whole() {
    let o = cast("Probe Reveal Five", 4);
    assert_eq!((o.bears_in_hand, o.graveyard, o.library, o.revealed), (3, 7 + 1, 0, 10));
}

/// No creature cards: nothing is revealed and nothing moves.
#[test]
fn zero_reveals_nothing() {
    let o = cast("Probe Reveal None", 10);
    assert_eq!((o.bears_in_hand, o.graveyard, o.library, o.revealed), (0, 1, 16, 0));
}
