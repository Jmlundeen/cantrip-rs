//! Eluge, the Shoreless Sea where no printed card in the engine takes it: its power and toughness as a spell another player
//! than its owner controls ("you" is its controller, CR 109.5, on the stack too), and its cost reduction's "land" for a flooded
//! permanent that has stopped being a land. Test cards do it, beside the printed Eluge and every other card the engine has.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn spell_on_stack(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id))
}

/// "Gain control of target spell."
fn commandeer(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(spell) = item.target(0) {
        g.change_controller(spell, item.controller);
    }
    Ok(())
}

fn land(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Land))
}

/// "Target land becomes an artifact and loses its other card types."
fn unland(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(id) = item.target(0) {
        g.add_effect(item.object, id, EffectKind::SetTypes(TypeSet::of(&[CardType::Artifact])), Duration::Indefinite);
    }
    Ok(())
}

static COMMANDEER: CardDef = CardDef { name: "Probe Commandeer", types: TypeSet::of(&[CardType::Instant]), spell: spell(&[target(spell_on_stack)], commandeer), ..CardDef::BLANK };
static UNLAND: CardDef = CardDef { name: "Probe Unland", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[target(land)], unland), ..CardDef::BLANK };
/// An instant costing {4}{U} that does nothing: what Eluge's reduction is seen on.
static OPT: CardDef = CardDef {
    name: "Probe Opt",
    cost: ManaCost { generic: 4, symbols: [0, 1, 0, 0, 0, 0], ..ManaCost::ZERO },
    types: TypeSet::of(&[CardType::Instant]),
    spell: spell(&[], |_, _| Ok(())),
    ..CardDef::BLANK
};

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&COMMANDEER, &UNLAND, &OPT])))
}

/// What a seat does, in order, each as soon as its turn has come and it is offered: play a land, or cast a card (aimed at a
/// permanent named, or at the spell on top of the stack).
enum Step {
    Play(usize, u32, &'static str),
    Cast(usize, u32, &'static str, Option<&'static str>),
}

/// A game of the two hands (then Islands), played until every step is done, turn `until` has come and the stack is empty. Every question is shown to
/// `watch` first. Mana is paid with Moxen, then lands.
fn play(hands: [&[&str]; 2], steps: Vec<Step>, until: u32, mut watch: impl FnMut(&Game, &Request)) -> Game {
    let names = |list: &[&str]| -> Vec<CardId> { list.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect() };
    let deck = |hand: &[&str]| {
        let mut deck = names(hand);
        deck.extend(names(&vec!["Island"; 30 - hand.len()]));
        deck
    };
    let mut game = Game::new(table(), &Setup { decks: vec![deck(hands[0]), deck(hands[1])], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let mut steps = steps.into_iter().peekable();
    for _ in 0..3000 {
        game.run();
        if steps.peek().is_none() && game.state.turn.number >= until && game.state.stack.is_empty() && game.state.triggers.is_empty() {
            return game;
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended on turn {} with {:?} left: {:?}", game.state.turn.number, steps.peek().map(|s| match s { Step::Play(_, _, n) | Step::Cast(_, _, n, _) => *n }), game.state.outcome) };
        watch(&game, &request);
        let seat = request.player.0 as usize;
        let turn = game.state.turn.number;
        let answer = match &request.kind {
            RequestKind::Priority { options } => {
                let option = match steps.peek() {
                    Some(&Step::Play(s, at, name)) if s == seat && turn >= at => {
                        options.iter().find(|o| o.kind == ActionKind::PlayLand && o.source.is_some_and(|c| game.name(c) == name))
                    }
                    Some(&Step::Cast(s, at, name, _)) if s == seat && turn >= at => {
                        options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|c| game.name(c) == name))
                    }
                    _ => None,
                };
                match option {
                    Some(option) => {
                        let aimed = match steps.next() {
                            Some(Step::Cast(_, _, _, aimed)) => aimed,
                            _ => None,
                        };
                        let mut action = Action::of(option);
                        if let Some(aimed) = aimed
                            && option.targets_required > 0
                        {
                            let target = if aimed == "stack" {
                                Target::Object(game.state.stack.last().expect("a spell").object)
                            } else if aimed == "flooded" {
                                Target::Object(*game.state.battlefield().iter().find(|&&id| game.state.obj(id).counters(CounterKind::Flood) > 0).expect("a flooded land"))
                            } else {
                                Target::Object(*game.state.battlefield().iter().find(|&&id| game.name(id) == aimed).expect("the target"))
                            };
                            action.targets = vec![target];
                        }
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::PayCosts { options, .. } => {
                let mox = options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source.is_some_and(|s| game.name(s).starts_with("Mox")));
                let land = options.iter().find(|o| o.kind == ActionKind::ActivateAbility);
                let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting);
                Answer::Act(Action::of(confirm.or(mox).or(land).expect("mana to pay with, or the payment to confirm")))
            }
            RequestKind::ChooseTarget { candidates, .. } => Answer::Targets(vec![candidates[0]]),
            RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..sources.len() as u8).collect()),
            // The newest cards, so a card planned for a later turn stays.
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().rev().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the game went on too long");
}

/// Bot1 (one Island) casts Eluge on turn 5; Bot2 (three Islands) takes the spell with Probe Commandeer. On the stack under
/// Bot2 it counts Bot2's Islands (CR 109.5: "you" is its controller), a 3/3, and it enters under Bot2 a 3/3.
#[test]
fn eluge_as_a_spell_another_player_controls_counts_that_players_islands() {
    let mut on_stack: Vec<(PlayerId, i16)> = Vec::new();
    let game = play(
        [&["Island", "Mox Sapphire", "Mox Sapphire", "Mox Sapphire", "Eluge, the Shoreless Sea"], &["Island", "Island", "Island", "Probe Commandeer"]],
        vec![
            Step::Play(0, 1, "Island"),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Play(1, 2, "Island"),
            Step::Play(1, 4, "Island"),
            Step::Play(1, 6, "Island"),
            Step::Cast(0, 7, "Eluge, the Shoreless Sea", None),
            Step::Cast(1, 7, "Probe Commandeer", Some("stack")),
        ],
        7,
        |game, _| {
            if let Some(item) = game.state.stack.iter().find(|i| i.kind == StackKind::Spell && game.name(i.object) == "Eluge, the Shoreless Sea") {
                let seen = (item.controller, game.chars(item.object).power.unwrap_or(-1));
                if on_stack.last() != Some(&seen) {
                    on_stack.push(seen);
                }
            }
        },
    );
    assert_eq!(on_stack, [(PlayerId(0), 1), (PlayerId(1), 3)], "Eluge on the stack, under Bot1 and then under Bot2");
    let eluge = *game.state.battlefield().iter().find(|&&id| game.name(id) == "Eluge, the Shoreless Sea").expect("Eluge");
    assert_eq!((game.state.obj(eluge).controller, game.chars(eluge).power), (PlayerId(1), Some(3)));
}

/// Eluge's flood counter is on Bot1's first Island, so the first instant Bot1 would cast each turn, Probe Opt ({4}{U}), costs {U}
/// less: {4}. On turn 4 Bot2's Probe Unland makes the flooded Island an artifact - still flooded, but no land (Eluge lives on as a
/// 1/1 with Bot1's second Island) - so on turn 5 the reduction is none and the Opt costs {4}{U} (CR 601.2f): its total as the
/// table works it out for the cast, each time Bot1 has priority with Eluge out and nothing on the stack.
#[test]
fn eluges_reduction_counts_no_flooded_permanent_that_is_no_land() {
    let mut totals: Vec<u32> = Vec::new();
    play(
        [&["Island", "Mox Sapphire", "Mox Sapphire", "Mox Sapphire", "Eluge, the Shoreless Sea", "Probe Opt", "Island"], &["Probe Unland"]],
        vec![
            Step::Play(0, 1, "Island"),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Cast(0, 1, "Mox Sapphire", None),
            Step::Cast(0, 1, "Eluge, the Shoreless Sea", None),
            Step::Play(0, 3, "Island"),
            Step::Cast(1, 4, "Probe Unland", Some("flooded")),
        ],
        6,
        |game, request| {
            let eluge = game.state.battlefield().iter().any(|&id| game.name(id) == "Eluge, the Shoreless Sea");
            if request.player != PlayerId(0) || !eluge || !game.state.stack.is_empty() || !matches!(request.kind, RequestKind::Priority { .. }) {
                return;
            }
            let Some(&opt) = game.state.zone(Zone::hand(PlayerId(0))).iter().find(|&&c| game.name(c) == "Probe Opt") else {
                panic!("no Opt on turn {}: hand {:?}, graveyard {:?}", game.state.turn.number, game.state.zone(Zone::hand(PlayerId(0))).iter().map(|&c| game.name(c)).collect::<Vec<_>>(), game.state.zone(Zone::graveyard(PlayerId(0))).iter().map(|&c| game.name(c)).collect::<Vec<_>>())
            };
            let total = cast_total(game, PlayerId(0), &ActionOption::on(ActionKind::CastSpell, opt, 0)).map_or(5, |(_, total, _)| total.mana_value());
            if totals.last() != Some(&total) {
                totals.push(total);
            }
        },
    );
    assert_eq!(totals, [4, 5], "the Opt's total before the Island stopped being a land, and after");
}
