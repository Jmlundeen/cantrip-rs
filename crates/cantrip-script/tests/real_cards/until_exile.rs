//! The end of an "until" exile (CR 610.3c): Palace Jailer's creature returns when an opponent becomes the monarch, as the
//! second one-shot effect the rules create immediately after the event - no triggered ability, so nothing that stops abilities
//! triggering stops it (`DelayedWhen::UntilEvent` in `settle::collect_batch`). The only "until" event in the engine is becoming
//! the monarch, which no printed card stops, so a test card that stops every triggered ability, and one that makes its caster
//! the monarch, do here, beside the printed Palace Jailer.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

/// "Triggered abilities don't trigger."
fn nothing_triggers(_: &dyn Query, _: &Asker, _: &Event, _: &Triggering) -> bool {
    true
}

/// A zero-cost artifact: "Triggered abilities don't trigger."
static HUSH: CardDef = CardDef { name: "Probe Hush", types: TypeSet::of(&[CardType::Artifact]), trigger_rules: &[nothing_triggers], ..CardDef::BLANK };

fn crown(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.become_monarch(item.controller);
    Ok(())
}

/// A zero-cost sorcery: "You become the monarch."
static CROWN: CardDef = CardDef { name: "Probe Crown", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], crown), ..CardDef::BLANK };

fn this_enters(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if *new == source && to.is(ZoneKind::Battlefield) => Some(Fired { subject: Target::None, data: [0, 0] }),
        _ => None,
    }
}

fn gain_one(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

/// A zero-cost 1/1: "When this creature enters, you gain 1 life." What the Hush is seen to stop.
static HERALD: CardDef = CardDef {
    name: "Probe Herald",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[triggered(when(EventMask::ZONE_CHANGE, this_enters), gain_one, "When this creature enters, you gain 1 life.")],
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&HUSH, &CROWN, &HERALD])))
}

/// Turn 2: Bot2 casts Ornithopter. Turn 3: Bot1 casts Palace Jailer (Black Lotus and Mox Pearl), becomes the monarch and exiles
/// the Ornithopter until an opponent becomes the monarch, then casts Probe Hush and a Probe Herald, whose enters ability the
/// Hush stops. Turn 4: Bot2 casts Probe Crown, and the Ornithopter comes back.
#[test]
fn an_until_exile_ends_under_a_rule_that_stops_triggered_abilities() {
    let names = |list: &[&str]| -> Vec<CardId> { list.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect() };
    let mut first = names(&["Black Lotus", "Mox Pearl", "Palace Jailer", "Probe Hush", "Probe Herald"]);
    first.extend(names(&["Plains"; 25]));
    let mut second = names(&["Ornithopter", "Probe Crown"]);
    second.extend(names(&["Island"; 28]));
    let mut game = Game::new(table(), &Setup { decks: vec![first, second], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let plan: [(u32, usize, &str); 7] =
        [(1, 0, "Black Lotus"), (1, 0, "Mox Pearl"), (2, 1, "Ornithopter"), (3, 0, "Palace Jailer"), (3, 0, "Probe Hush"), (3, 0, "Probe Herald"), (4, 1, "Probe Crown")];
    let mut steps = plan.iter().peekable();
    for _ in 0..2000 {
        game.run();
        if steps.peek().is_none() && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            break;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } if game.state.stack.is_empty() => match steps.peek() {
                Some(&&(turn, seat, name)) if game.state.turn.number >= turn && request.player.0 as usize == seat => {
                    match options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == name)) {
                        Some(option) => {
                            steps.next();
                            Answer::Act(Action::of(option))
                        }
                        None => Answer::Pass,
                    }
                }
                _ => Answer::Pass,
            },
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { options, .. } => {
                let lotus = options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s) == "Black Lotus"));
                let mox = options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s) == "Mox Pearl"));
                let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting);
                let mut action = Action::of(confirm.or(lotus).or(mox).expect("mana to pay with, or the payment to confirm"));
                if action.source.is_some_and(|s| game.name(s) == "Black Lotus") {
                    action.color = Some(Color::White);
                }
                Answer::Act(action)
            }
            RequestKind::ChooseTarget { candidates, .. } => Answer::Targets(vec![candidates[0]]),
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..sources.len() as u8).collect()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert!(steps.peek().is_none(), "the plan was not played out");
    // The positive control: the Herald, entering after the Hush, gained Bot1 nothing.
    assert_eq!(game.state.player(PlayerId(0)).life, 20, "the Hush stopped no triggered ability");
    assert!(game.state.player(PlayerId(1)).monarch, "Bot2 is not the monarch");
    let thopter = game.state.battlefield().iter().copied().find(|&id| game.name(id) == "Ornithopter");
    assert!(thopter.is_some_and(|id| game.state.obj(id).controller == PlayerId(1)), "the Ornithopter did not come back when Bot2 became the monarch");
}
