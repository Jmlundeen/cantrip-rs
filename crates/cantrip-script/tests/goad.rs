//! Goad (CR 701.15) as `cantrip_mechanics::goad` does it, over test-only cards: the paths the Undercity's Arena, the one goad
//! in the engine, never reaches. A goad for a printed duration ("goaded for the rest of the game", Dack Fayden, Helping Hand)
//! outlasts its goader's next turn; the same player goading a creature again has no effect, whatever either duration
//! (CR 701.15d); and the goader is the controller of the ability that goaded, even once its source has changed hands
//! (CR 701.15b).

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::{goad, targets};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn the_creature(item: &StackItem) -> ObjectId {
    match item.target(0) {
        Target::Object(id) => id,
        other => panic!("a creature, not {other:?}"),
    }
}

/// "Goad target creature."
fn goad_it(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    goad::goad(g, item, the_creature(item));
    Ok(())
}

/// "Goad target creature. It's goaded for the rest of the game."
fn goad_it_forever(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    goad::goad_for(g, item, the_creature(item), Duration::Indefinite);
    Ok(())
}

/// "Target opponent gains control of this permanent. Then goad target creature for the rest of the game."
fn hand_off_then_goad(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    g.gain_control(item.object, item.object, P1, Duration::Indefinite);
    g.end_sentence();
    goad::goad_for(g, item, the_creature(item), Duration::Indefinite);
    Ok(())
}

static CREATURE: [TargetSpec; 1] = [targets::CREATURE];

const fn probe(name: &'static str, effect: fn(&mut dyn Rules, &StackItem) -> Result<(), Suspend>) -> CardDef {
    CardDef { name, types: TypeSet::of(&[CardType::Instant]), spell: spell(&CREATURE, effect), ..CardDef::BLANK }
}

static GOAD: CardDef = probe("Probe Goad", goad_it);
static GOAD_FOREVER: CardDef = probe("Probe Goad Forever", goad_it_forever);
static HANDOFF_ABILITY: [AbilityDef; 1] = [ability(cost("{0}"), hand_off_then_goad, "{0}: Hand this off, then goad.").targets(&CREATURE)];
static HANDOFF: CardDef = CardDef { name: "Probe Handoff", types: TypeSet::of(&[CardType::Artifact]), abilities: &HANDOFF_ABILITY, ..CardDef::BLANK };
static BEAR: CardDef = CardDef { name: "Probe Bear", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), ..CardDef::BLANK };
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&GOAD, &GOAD_FOREVER, &HANDOFF, &BEAR, &FILLER]))
}

fn put(game: &mut Game, name: &str, controller: PlayerId) -> ObjectId {
    let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), controller, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, id);
    id
}

/// A two-player game: the first seat holds the named spells and has the Handoff; the second has a Bear.
fn deal(hand: &[&str]) -> (Game, ObjectId) {
    let filler = table().find("Filler").unwrap();
    let mut deck: Vec<_> = hand.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect();
    deck.extend(std::iter::repeat(filler).take(30 - deck.len()));
    let other = vec![filler; 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, other], seed: 1, shuffle: false, ..Setup::default() });
    game.verify = true;
    put(&mut game, "Probe Handoff", P0);
    let bear = put(&mut game, "Probe Bear", P1);
    game.state.see_statics();
    (game, bear)
}

/// The goads on `creature`: who goaded it, and for how long.
fn goads(game: &Game, creature: ObjectId) -> Vec<(PlayerId, Duration)> {
    game.state.effects.iter().filter(|e| e.affected == creature && e.kind == EffectKind::Goaded).map(|e| (e.controller, e.duration)).collect()
}

/// The first seat casts the named spells (or activates the named permanent) in order on turn 1 at the Bear, each resolving
/// before the next; then everyone passes, and the goaded Bear attacks as it must, until `turn` begins. What `goads` says
/// once all have resolved, and as `turn` begins.
fn play(names: &[&str], turn: u32) -> (Vec<(PlayerId, Duration)>, Vec<(PlayerId, Duration)>, Game) {
    let (mut game, bear) = deal(names);
    let mut next = 0;
    let mut resolved = None;
    for _ in 0..2000 {
        game.run();
        if next == names.len() && resolved.is_none() && game.state.stack.is_empty() {
            resolved = Some(goads(&game, bear));
        }
        if game.state.turn.number == turn {
            let after = resolved.expect("all resolved");
            return (after, goads(&game, bear), game);
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == P0 && game.state.stack.is_empty() && next < names.len() => {
                let option = options
                    .iter()
                    .find(|o| matches!(o.kind, ActionKind::CastSpell | ActionKind::ActivateAbility) && o.source.is_some_and(|s| game.name(s) == names[next]))
                    .unwrap_or_else(|| panic!("{} can't be used", names[next]));
                let mut action = Action::of(option);
                action.targets = vec![Target::Object(bear)];
                next += 1;
                Answer::Act(action)
            }
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { requirements, .. } => Answer::Attack(requirements.suggested.clone()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("turn {turn} never began");
}

/// "Until your next turn" (CR 701.15a): goaded by the first seat until its turn 3.
#[test]
fn a_goad_lasts_until_its_goaders_next_turn() {
    let (after, later, _) = play(&["Probe Goad"], 3);
    assert_eq!(after, vec![(P0, Duration::UntilNextTurnOf(P0))]);
    assert_eq!(later, vec![]);
}

/// "Goaded for the rest of the game" (CR 611.2a): still goaded on the goader's next turns.
#[test]
fn a_goad_for_the_rest_of_the_game_outlasts_its_goaders_next_turn() {
    let (after, later, _) = play(&["Probe Goad Forever"], 5);
    assert_eq!(after, vec![(P0, Duration::Indefinite)]);
    assert_eq!(later, vec![(P0, Duration::Indefinite)]);
}

/// "Once a player has goaded a creature, the same player goading it again has no effect" (CR 701.15d): a longer goad from
/// the same player neither adds a requirement nor extends the first, which ends with the goader's next turn.
#[test]
fn the_same_player_goading_again_has_no_effect() {
    let (after, later, _) = play(&["Probe Goad", "Probe Goad Forever"], 3);
    assert_eq!(after, vec![(P0, Duration::UntilNextTurnOf(P0))]);
    assert_eq!(later, vec![]);
}

/// The goader is the controller of the ability that caused the goad (CR 701.15b), not whoever controls its source by the
/// time the creature is goaded: the second seat, which now controls the Handoff, is not the goader of its own Bear.
#[test]
fn the_goader_is_the_abilitys_controller_once_its_source_has_changed_hands() {
    let (after, _, game) = play(&["Probe Handoff"], 2);
    let handoff = game.state.battlefield().iter().copied().find(|&id| game.name(id) == "Probe Handoff").expect("the Handoff");
    assert_eq!(game.state.obj(handoff).controller, P1);
    assert_eq!(after, vec![(P0, Duration::Indefinite)]);
}
