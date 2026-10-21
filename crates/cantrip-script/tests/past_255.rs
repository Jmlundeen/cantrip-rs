//! Numbers past 255 that the rules set no limit on, over test cards: a spell with "any number of target creatures" chooses
//! more than 250 of them (CR 115.1d) and every one is its target; "X target creatures" is X targets whatever X is (CR 601.2c);
//! and a spell of mana value 260 is recorded as cast with mana value 260 (CR 202.3, 601.2i), which what asks about past casts
//! reads. No printed card in the engine reaches such numbers on today's boards, so test cards do here.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Creature))
}

/// You gain 1 life for each of its targets that is still legal.
fn gain_per_target(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let legal = (0..item.target_count as usize).filter(|&slot| g.target_legal(item, slot)).count();
    g.gain_life(item.controller, legal as i32);
    Ok(())
}

fn nothing(_: &mut dyn Rules, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// A zero-cost sorcery: "Choose any number of target creatures. You gain 1 life for each of them."
static VOLLEY: CardDef = CardDef {
    name: "Probe Volley",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[up_to(TargetSpec::ANY_NUMBER, creature)], gain_per_target),
    ..CardDef::BLANK
};

/// A sorcery costing {250}{W}{W}{W}{W}{W}{W}{W}{W}{W}{W}, with no effect.
static SURGE: CardDef = CardDef {
    name: "Probe Surge",
    types: TypeSet::of(&[CardType::Sorcery]),
    cost: ManaCost { generic: 250, symbols: [10, 0, 0, 0, 0, 0], ..ManaCost::ZERO },
    spell: spell(&[], nothing),
    ..CardDef::BLANK
};

/// A land: "{T}: Add {W}{W}{W}{W}{W}{W}{W}{W}{W}{W} and 250 {C}."
static WELL: CardDef = CardDef {
    name: "Probe Well",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    abilities: &[mana_ability(TAP, Adds { pool: ManaPool([10, 0, 0, 0, 0, 250]), ..add_n(Mana::C, 0) }, "{T}: Add ten {W} and 250 {C}.")],
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
    TABLE.get_or_init(|| CardTable::new([&VOLLEY, &SURGE, &WELL, &BEAR, &FILLER]))
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

/// With 300 creatures on the battlefield, Probe Volley may target every one (CR 115.1d): it is asked for up to 300, all 300
/// are its targets, and it gains 300 life as each is still legal.
#[test]
fn any_number_of_targets_is_more_than_250() {
    let mut game = game("Probe Volley");
    let bear = table().find("Probe Bear").unwrap();
    for _ in 0..300 {
        let id = game.state.push_obj(Obj::new(bear, PlayerId(1), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
    }
    let mut asked = None;
    let mut held = 0;
    cast(&mut game, "Probe Volley", |game, request| match &request.kind {
        RequestKind::ChooseTarget { candidates, count, .. } => {
            asked = Some(*count);
            Some(Answer::Targets(candidates.clone()))
        }
        RequestKind::Priority { .. } if !game.state.stack.is_empty() => {
            held = game.state.targets(game.state.stack.last().unwrap()).len();
            None
        }
        _ => None,
    });
    assert_eq!(asked, Some(300), "not asked for up to every creature");
    assert_eq!(held, 300, "the spell does not hold every target chosen");
    assert_eq!(game.state.player(PlayerId(0)).life, 320, "not every target was a target as it resolved");
}

/// "X target creatures" is X of them, whatever X is (CR 601.2c).
#[test]
fn a_counted_slot_is_as_many_as_counted() {
    let slot = up_to(1, creature).counted();
    assert_eq!(slot.bounds(300), (300, 300));
    assert_eq!(up_to(1, creature).between(0, TargetSpec::COUNTED).bounds(1000), (0, 1000));
}

/// Probe Surge cast, paid with Probe Well's 250 colorless and 10 white mana: the history records its mana value as 260 (CR 202.3).
#[test]
fn a_spell_of_mana_value_260_is_cast_with_mana_value_260() {
    let mut game = game("Probe Surge");
    let well = game.state.push_obj(Obj::new(table().find("Probe Well").unwrap(), PlayerId(0), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, well);
    cast(&mut game, "Probe Surge", |_, request| match &request.kind {
        RequestKind::PayCosts { options, .. } => {
            let option = options
                .iter()
                .find(|o| o.kind == ActionKind::ConfirmCasting)
                .or_else(|| options.iter().find(|o| o.kind == ActionKind::ActivateAbility && o.source == Some(well)))
                .or_else(|| options.iter().find(|o| o.kind == ActionKind::AssignMana));
            Some(Answer::Act(Action::of(option.unwrap_or_else(|| panic!("the mana to pay with: {request:?}")))))
        }
        _ => None,
    });
    let values: Vec<u32> = game.log.iter().filter_map(|r| match r.event {
        Event::Cast(c) => Some(c.value),
        _ => None,
    }).collect();
    assert_eq!(values, [260]);
}
