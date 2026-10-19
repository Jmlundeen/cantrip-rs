//! "For as long as it has a [kind] counter on it" (CR 611.2b, `Duration::WhileCounter`): the paths no printed card in the
//! engine reaches, played on a table of test-only cards. Eluge, the Shoreless Sea's scenarios prove the effect lasts while the
//! counter does and ends with it; but every flood counter a printed card puts comes with an effect of its own (Eluge, Aquitect's
//! Will, Quicksilver Fountain, Xolatoyac), so no scenario can tell an ended effect that came back from a new one. These are:
//!
//! - a counter of the kind put on again after the last one came off does not bring the ended effect back (CR 611.2b: an effect
//!   that has ended stays ended);
//! - the effect never begins on an object without such a counter.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn land(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.chars(id).is(CardType::Land) && q.obj(id).zone.is(ZoneKind::Battlefield))
}

fn the_land(item: &StackItem) -> ObjectId {
    match item.targets()[0] {
        Target::Object(id) => id,
        other => panic!("a land, not {other:?}"),
    }
}

fn island_while_flooded(g: &mut dyn Rules, item: &StackItem) {
    g.add_effect(item.object, the_land(item), EffectKind::AddSubtype(Subtype::Island), Duration::WhileCounter(CounterKind::Flood));
}

/// Put a flood counter on target land. It's an Island in addition to its other types for as long as it has a flood counter on it.
fn flood(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(the_land(item), CounterKind::Flood, 1);
    island_while_flooded(g, item);
    Ok(())
}

/// Target land is an Island for as long as it has a flood counter on it (no counter is put).
fn bare(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    island_while_flooded(g, item);
    Ok(())
}

/// Remove all flood counters from target land.
fn drain(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let land = the_land(item);
    let n = g.obj(land).counters(CounterKind::Flood);
    g.remove_counters(land, CounterKind::Flood, n);
    Ok(())
}

/// Put a flood counter on target land.
fn refill(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(the_land(item), CounterKind::Flood, 1);
    Ok(())
}

static LAND: [TargetSpec; 1] = [target(land)];

const fn probe(name: &'static str, effect: fn(&mut dyn Rules, &StackItem) -> Result<(), Suspend>) -> CardDef {
    CardDef { name, types: TypeSet::of(&[CardType::Instant]), spell: spell(&LAND, effect), ..CardDef::BLANK }
}

static FLOOD: CardDef = probe("Probe Flood", flood);
static BARE: CardDef = probe("Probe Bare", bare);
static DRAIN: CardDef = probe("Probe Drain", drain);
static REFILL: CardDef = probe("Probe Refill", refill);

/// A land with no land type, to fill hands and libraries.
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&FLOOD, &BARE, &DRAIN, &REFILL, &FILLER]))
}

/// A two-player game: the first seat's hand is a Filler, then the named cards; the rest are Fillers.
fn deal(hand: &[&str]) -> Game {
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![filler];
    deck.extend(hand.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))));
    deck.extend(std::iter::repeat(filler).take(30 - deck.len()));
    let other = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, other], seed: 1, shuffle: false, opening_chooser: None, sideboards: Vec::new(), basic_lands_outside: Vec::new() });
    game.verify = true;
    game.run();
    game
}

/// The first seat plays its Filler, then casts the named spells in order at its Filler, each resolving before the next; `after`
/// is called once each has resolved.
fn play(game: &mut Game, casts: &[&str], mut after: impl FnMut(&Game, &str)) {
    let mut played = false;
    let mut next = 0;
    let mut waiting: Option<&str> = None;
    for _ in 0..400 {
        game.run();
        if let Some(name) = waiting
            && game.state.stack.is_empty()
        {
            after(game, name);
            waiting = None;
            if next == casts.len() {
                return;
            }
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && waiting.is_none() => {
                let land = options.iter().find(|o| o.kind == ActionKind::PlayLand);
                let cast = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| next < casts.len() && game.name(s) == casts[next]));
                match (played, land, cast) {
                    (false, Some(option), _) => {
                        played = true;
                        Answer::Act(Action::of(option))
                    }
                    (true, _, Some(option)) => {
                        let mut action = Action::of(option);
                        action.targets = vec![option.target_candidates[0]];
                        waiting = Some(casts[next]);
                        next += 1;
                        Answer::Act(action)
                    }
                    _ => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the spells did not all resolve");
}

/// The first seat's land on the battlefield: whether it is an Island, and its flood counters.
fn the_filler(game: &Game) -> (bool, i16) {
    let id = game.state.battlefield().iter().copied().find(|&id| game.state.obj(id).controller == PlayerId(0)).expect("the land");
    (game.chars(id).has_subtype(Subtype::Island), game.state.obj(id).counters(CounterKind::Flood))
}

/// CR 611.2b: once the last flood counter comes off, the effect has ended, and a flood counter put on the land again does not
/// make it an Island; a new effect of the same words does.
#[test]
fn a_counter_put_back_after_the_last_came_off_does_not_restart_the_effect() {
    let mut game = deal(&["Probe Flood", "Probe Drain", "Probe Refill", "Probe Flood"]);
    let mut seen = Vec::new();
    play(&mut game, &["Probe Flood", "Probe Drain", "Probe Refill", "Probe Flood"], |g, name| seen.push((name.to_string(), the_filler(g))));
    let expected: Vec<(String, (bool, i16))> = [
        ("Probe Flood", (true, 1)),
        ("Probe Drain", (false, 0)),
        ("Probe Refill", (false, 1)),
        ("Probe Flood", (true, 2)),
    ]
    .into_iter()
    .map(|(n, s)| (n.to_string(), s))
    .collect();
    assert_eq!(seen, expected);
    // The ended effect is gone from the game, not dormant: only the second Probe Flood's remains.
    let lasting = game.state.effects.iter().filter(|e| e.duration == Duration::WhileCounter(CounterKind::Flood)).count();
    assert_eq!(lasting, 1);
}

/// CR 611.2b: "for as long as it has a flood counter on it" never begins on a land without one, so a flood counter put on it
/// afterwards makes it nothing.
#[test]
fn the_effect_never_begins_on_an_object_without_the_counter() {
    let mut game = deal(&["Probe Bare", "Probe Refill"]);
    let mut seen = Vec::new();
    play(&mut game, &["Probe Bare", "Probe Refill"], |g, name| seen.push((name.to_string(), the_filler(g))));
    assert_eq!(seen, vec![("Probe Bare".to_string(), (false, 0)), ("Probe Refill".to_string(), (false, 1))]);
    assert!(game.state.effects.iter().all(|e| e.duration != Duration::WhileCounter(CounterKind::Flood)));
}
