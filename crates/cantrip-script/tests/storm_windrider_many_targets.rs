//! Storm, Windrider gives flying to every creature a spell targeted as it was cast, however many: a test spell, "choose any
//! number of target creatures", targets 40 (past the 32 a bit set would hold) and all 40 gain flying (CR 115.1d, 611.2a).

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Creature))
}

fn nothing(_: &mut dyn Rules, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// A zero-cost sorcery: "Choose any number of target creatures."
static VOLLEY: CardDef = CardDef {
    name: "Probe Volley",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[up_to(TargetSpec::ANY_NUMBER, creature)], nothing),
    ..CardDef::BLANK
};

static BEAR: CardDef = CardDef {
    name: "Probe Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// A land to fill hands and libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&VOLLEY, &BEAR, &FILLER])))
}

/// A game in which Bot1 holds `card` on turn 1.
fn game(card: &str) -> Game {
    let names = |list: &[&str]| -> Vec<CardId> { list.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect() };
    let mut first = names(&[card]);
    first.extend(names(&["Filler"; 29]));
    let mut game = Game::new(table(), &Setup { decks: vec![first, names(&["Filler"; 30])], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    game
}

/// Bot1 casts `card` in its first main phase, answering what it is asked by `answer` (None for the plain answer), until it
/// has resolved.
fn cast(game: &mut Game, card: &str, mut answer: impl FnMut(&Game, &Request) -> Option<Answer>) {
    let mut cast = false;
    for _ in 0..2000 {
        game.run();
        let settled = game.state.stack.is_empty() && game.state.triggers.is_empty() && matches!(game.pending().map(|r| &r.kind), Some(RequestKind::Priority { .. }));
        if cast && settled {
            return;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let reply = answer(game, &request).unwrap_or_else(|| match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && !cast && game.state.turn.step() == Step::PrecombatMain => {
                let option = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == card));
                cast = true;
                Answer::Act(Action::of(option.unwrap_or_else(|| panic!("{card} is not offered"))))
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        });
        game.answer(request.id, request.player, reply).unwrap();
    }
    panic!("{card} was not cast and resolved");
}

/// With Storm, Windrider on the battlefield and 40 creatures there, Probe Volley targets all 40: every one gains flying, the
/// 33rd and the rest included.
#[test]
fn every_creature_among_forty_targets_gains_flying() {
    let mut game = game("Probe Volley");
    let storm = game.state.push_obj(Obj::new(table().find("Storm, Windrider").unwrap(), PlayerId(0), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, storm);
    let bear = table().find("Probe Bear").unwrap();
    let mut bears = Vec::new();
    for _ in 0..40 {
        let id = game.state.push_obj(Obj::new(bear, PlayerId(1), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        bears.push(id);
    }
    cast(&mut game, "Probe Volley", |_, request| match &request.kind {
        RequestKind::ChooseTarget { candidates, .. } => Some(Answer::Targets(candidates.clone())),
        _ => None,
    });
    let flying = bears.iter().filter(|&&id| game.chars(id).has(Keyword::Flying)).count();
    assert_eq!(flying, 40, "not every target gained flying");
}
