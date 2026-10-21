//! A player conceding while a spell resolves in a game of three (CR 104.3a, 800.4a): they leave the game at the point they
//! conceded, after what had already happened - the resolution is not wound back to before it, and answers given in it stay
//! the answers of the players who gave them (`Game::concede`, `Frame::conceded`). Test cards stand in for Price of Fame
//! ("destroy target creature; surveil 2") and Council's Judgment's vote, beside the printed Blood Artist.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Creature))
}

fn fame(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = cantrip_mechanics::targets::object(g, item, 0) {
        g.destroy(creature);
    }
    cantrip_mechanics::surveil(g, item.controller, item.object, 2)
}

/// A zero-cost sorcery: "Destroy target creature. Surveil 2." (Price of Fame, less its cost.)
static FAME: CardDef = CardDef { name: "Probe Fame", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[target(creature)], fame), ..CardDef::BLANK };

fn council(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let candidates: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.controller(id) != you).collect();
    let votes = cantrip_mechanics::vote::for_permanents(g, item.object, you, &candidates)?;
    g.exile_all(&cantrip_mechanics::vote::most_votes(&candidates, &votes));
    Ok(())
}

/// A zero-cost sorcery: "Will of the council - starting with you, each player votes for a permanent you don't control. Exile
/// each permanent with the most votes or tied for most votes." (Council's Judgment, less its cost.)
static COUNCIL: CardDef = CardDef { name: "Probe Council", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], council), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&FAME, &COUNCIL])))
}

/// A game of three, Bot1 holding `card` on turn 1, with `permanents` on the battlefield. Bot1 casts `card` in its main phase;
/// `answer` answers every other question (None for a pass), and may make P1 concede instead (`true`), once.
fn play(card: &str, permanents: &[(&str, PlayerId)], mut answer: impl FnMut(&Game, &Request) -> (Option<Answer>, bool)) -> (Game, Vec<ObjectId>) {
    let names = |list: &[&str]| -> Vec<CardId> { list.iter().map(|n| table().find(n).unwrap_or_else(|| panic!("no card {n}"))).collect() };
    let mut first = names(&[card]);
    first.extend(names(&["Plains"; 29]));
    let deck = || names(&["Plains"; 30]);
    let mut game = Game::new(table(), &Setup { decks: vec![first, deck(), deck()], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    let ids: Vec<ObjectId> = permanents
        .iter()
        .map(|&(name, owner)| {
            let id = game.state.push_obj(Obj::new(table().find(name).unwrap(), owner, Zone::BATTLEFIELD));
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        })
        .collect();
    let (mut cast, mut conceded) = (false, false);
    for _ in 0..500 {
        game.run();
        let Some(request) = game.pending().cloned() else { break };
        if cast && game.state.stack.is_empty() && game.state.triggers.is_empty() && matches!(request.kind, RequestKind::Priority { .. }) {
            break;
        }
        let reply = match &request.kind {
            RequestKind::Priority { options } if request.player == P0 && !cast && game.state.turn.step() == Step::PrecombatMain => {
                let option = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source.is_some_and(|s| game.name(s) == card));
                cast = true;
                let option = option.unwrap_or_else(|| panic!("{card} is not offered"));
                let mut action = Action::of(option);
                // A target announced with the cast: P1's creature.
                if option.targets_required > 0 {
                    action.targets = option.target_candidates.iter().copied().filter(|t| t.object().is_some_and(|id| game.state.obj(id).controller == P1)).take(1).collect();
                }
                Some(Answer::Act(action))
            }
            RequestKind::Priority { .. } => Some(Answer::Pass),
            _ => {
                let (reply, concede) = answer(&game, &request);
                if concede && !conceded {
                    conceded = true;
                    game.concede(P1);
                    continue;
                }
                reply
            }
        };
        let reply = reply.unwrap_or_else(|| panic!("unexpected question {:?}", request.kind));
        game.answer(request.id, request.player, reply).unwrap();
    }
    assert!(cast, "{card} was not cast");
    assert!(conceded, "P1 never conceded");
    (game, ids)
}

/// Probe Fame destroys P1's Ornithopter with P0's Blood Artist out, and P1 concedes while P0 surveils. The Ornithopter stays destroyed - the
/// spell does not lose its target - Blood Artist's dies trigger stands (P2 loses 1, P0 gains 1), and the surveil P0 was making
/// is asked again and made.
#[test]
fn a_concession_during_a_resolution_comes_after_what_it_had_done() {
    let mut surveils = 0;
    let (game, ids) = play("Probe Fame", &[("Blood Artist", P0), ("Ornithopter", P1)], |game, request| match &request.kind {
        // The Ornithopter for Probe Fame; P2 for Blood Artist's trigger.
        RequestKind::ChooseTarget { candidates, .. } => {
            let bear = Target::Object(ids_bear(game));
            (Some(Answer::Targets(vec![if candidates.contains(&bear) { bear } else { Target::Player(P2) }])), false)
        }
        RequestKind::DistributeCards { cards, .. } => {
            surveils += 1;
            (Some(Answer::Placements { placements: vec![Vec::new(), cards.clone()], shuffle: false }), surveils == 1)
        }
        RequestKind::OrderTriggers { sources, .. } => (Some(Answer::Order((0..sources.len() as u8).collect())), false),
        _ => (None, false),
    });
    assert!(!game.state.battlefield().contains(&ids[1]), "the Ornithopter is still there");
    assert!(game.state.player(P1).lost, "P1 did not leave");
    assert_eq!(surveils, 2, "the surveil was not asked again after the concession");
    assert!(game.log.iter().any(|r| matches!(r.event, Event::Surveilled { .. })), "the surveil was not made");
    assert!(!game.log.iter().any(|r| matches!(r.event, Event::Countered { .. })), "the spell lost its target");
    assert_eq!(game.state.player(P2).life, 19, "Blood Artist's trigger did not make P2 lose 1");
    assert_eq!(game.state.player(P0).life, 21, "Blood Artist's trigger did not make P0 gain 1");
    let binned = game.state.zone(Zone::graveyard(P0)).iter().filter(|&&id| game.name(id) == "Plains").count();
    assert_eq!(binned, 2, "the surveil did not put both cards into the graveyard");
}

fn ids_bear(game: &Game) -> ObjectId {
    game.state.battlefield().iter().copied().find(|&id| game.name(id) == "Ornithopter").unwrap_or(ObjectId::NONE)
}

/// P0, P1 and P2 vote in turn on Probe Council between two of P2's Ornithopters; P1 votes for the second and concedes while P2 is
/// voting. P1's vote was cast before they left and counts; P2's is asked again of P2 and given to the second as well - so
/// the second alone has the most votes and is exiled. Were P1's vote lost, or replayed as P2's, the two would tie.
#[test]
fn a_concession_keeps_the_answers_given_before_it_with_the_players_who_gave_them() {
    let mut p2_asked = 0;
    let (game, ids) = play("Probe Council", &[("Ornithopter", P2), ("Ornithopter", P2)], |game, request| match &request.kind {
        RequestKind::ChooseCards { eligible, .. } => {
            let _ = game;
            // The Ornithopters by age: the first put onto the battlefield, then the second.
            let (first, second) = (*eligible.iter().min().unwrap(), *eligible.iter().max().unwrap());
            match request.player {
                P0 => (Some(Answer::Cards(vec![first])), false),
                P1 => (Some(Answer::Cards(vec![second])), false),
                _ => {
                    p2_asked += 1;
                    (Some(Answer::Cards(vec![second])), p2_asked == 1)
                }
            }
        }
        _ => (None, false),
    });
    assert!(game.state.player(P1).lost, "P1 did not leave");
    assert!(game.state.battlefield().contains(&ids[0]), "the first Ornithopter was exiled: the votes tied");
    assert!(!game.state.battlefield().contains(&ids[1]), "the second Ornithopter, with two votes, stayed");
}
