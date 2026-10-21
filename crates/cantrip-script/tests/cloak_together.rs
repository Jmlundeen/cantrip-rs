//! Cloaking several cards (CR 701.58): those from a single library one at a time (CR 701.58e), any others - cards in a hand, a
//! face-down exile pile (Become Anonymous, Expose the Culprit) - onto the battlefield together (CR 110.2a), so they are one
//! event (CR 603.2c). No printed card in the engine cloaks several cards from anywhere but a library yet, so test cards do here:
//! a "one or more creatures you control enter" watcher triggers once for cards cloaked together and once for each card cloaked
//! from a library.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

/// "Whenever one or more creatures you control enter": a creature entering under the watcher's controller.
fn creature_you_control_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Some(Fired { subject: Target::Object(*new), data: [0, 0] })
        }
        _ => None,
    }
}

fn gain_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// A zero-cost enchantment: "Whenever one or more creatures you control enter, you gain 1 life."
static WITNESS: CardDef = CardDef {
    name: "Probe Witness",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, creature_you_control_enters), gain_one, "Whenever one or more creatures you control enter, you gain 1 life.")
        .once_per_batch()],
    ..CardDef::BLANK
};

/// "Cloak each other card in your hand."
fn cloak_hand(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let hand: Vec<ObjectId> = g.state().zone(Zone::hand(item.controller)).iter().copied().filter(|&c| c != item.object).collect();
    cantrip_mechanics::cloak::cloak(g, item.controller, &hand, false)?;
    Ok(())
}

/// "Cloak the top two cards of your library."
fn cloak_library(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let top: Vec<ObjectId> = g.state().zone(Zone::library(item.controller)).iter().copied().take(2).collect();
    cantrip_mechanics::cloak::cloak(g, item.controller, &top, false)?;
    Ok(())
}

static HAND_CLOAK: CardDef = CardDef { name: "Probe Hand Cloak", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], cloak_hand), ..CardDef::BLANK };
static LIBRARY_CLOAK: CardDef =
    CardDef { name: "Probe Library Cloak", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], cloak_library), ..CardDef::BLANK };
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&WITNESS, &HAND_CLOAK, &LIBRARY_CLOAK, &FILLER]))
}

/// Bot1 casts the Witness, then `cloaker`, with Fillers filling the rest of its hand and library. How many permanents it then
/// has face down, and its life.
fn cloaked_with(cloaker: &str) -> (usize, i32) {
    let find = |n: &str| table().find(n).unwrap();
    let mut first = vec![find("Probe Witness"), find(cloaker)];
    first.extend(std::iter::repeat_n(find("Filler"), 28));
    let second = vec![find("Filler"); 30];
    let mut game = Game::new(table(), &Setup { decks: vec![first, second], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut casts = ["Probe Witness", cloaker].into_iter().peekable();
    for _ in 0..200 {
        game.run();
        if casts.peek().is_none() && game.state.stack.is_empty() && game.state.triggers.is_empty() && !game.pending().is_some_and(|r| matches!(r.kind, RequestKind::OrderTriggers { .. })) {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && game.state.stack.is_empty() => {
                match casts.peek().and_then(|&n| options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == n))) {
                    Some(option) => {
                        casts.next();
                        Answer::Act(Action::of(option))
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..count_u32(sources.len())).collect()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    let face_down = game.state.battlefield().iter().filter(|&&id| game.state.obj(id).face_down).count();
    (face_down, game.state.player(PlayerId(0)).life)
}

/// The five Fillers left in Bot1's hand are cloaked together: one event, so the watcher triggers once.
#[test]
fn cards_cloaked_from_a_hand_enter_together() {
    assert_eq!(cloaked_with("Probe Hand Cloak"), (5, 21));
}

/// The top two cards of a library are cloaked one at a time (CR 701.58e): two events, so the watcher triggers twice.
#[test]
fn cards_cloaked_from_a_library_enter_one_at_a_time() {
    assert_eq!(cloaked_with("Probe Library Cloak"), (2, 22));
}
