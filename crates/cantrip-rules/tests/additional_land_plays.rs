//! "You may play an additional land this turn" (CR 305.2, 611.2c; `Game::additional_land_plays`): an effect a resolved
//! spell or ability makes for its controller, which adds to the number of lands they can play until the turn ends -
//! cumulatively with every other such effect and with the static abilities that do the same (CR 305.2), compared with the
//! lands already played this turn however that was (CR 305.2a/b). The cards here are probes, not card implementations:
//! Explore's and Journey of Discovery's effects, cast for nothing, and Icetill Explorer's static rule.

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game as RulesGame, Setup};
use cantrip_state::{Frame, FrameKind};

const ME: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

fn one_more(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.additional_land_plays(item, 1);
    Ok(())
}

fn two_more(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.additional_land_plays(item, 2);
    Ok(())
}

fn none_more(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.additional_land_plays(item, 0);
    Ok(())
}

/// "You may play an additional land this turn" (Explore's), an instant for nothing so it can be cast on any turn.
static ONE_MORE: CardDef = CardDef {
    name: "Extra Land Probe",
    oracle: true,
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef { targets: &[], effect: one_more }),
    ..CardDef::BLANK
};

/// "You may play up to two additional lands this turn" (Journey of Discovery's).
static TWO_MORE: CardDef = CardDef { name: "Two Extra Lands Probe", spell: Some(SpellDef { targets: &[], effect: two_more }), ..ONE_MORE };

/// "X additional lands" for an X of 0 (Nahiri's Lithoforming, no land sacrificed).
static NONE_MORE: CardDef = CardDef { name: "No Extra Land Probe", spell: Some(SpellDef { targets: &[], effect: none_more }), ..ONE_MORE };

/// "You may play an additional land on each of your turns" (Icetill Explorer's, Dryad of the Ilysian Grove's static).
static STATIC_MORE: CardDef = CardDef {
    name: "Extra Land Static Probe",
    types: TypeSet::of(&[CardType::Artifact]),
    play_rules: &[PlayRule::ExtraLandPlays(1)],
    ..CardDef::BLANK
};

static LAND: CardDef = CardDef { name: "Extra Land Probe Land", oracle: true, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Extra Land Probe Filler", has_cost: false, ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&ONE_MORE, &TWO_MORE, &NONE_MORE, &STATIC_MORE, &LAND, &FILLER]))
}

/// A game in the first player's precombat main phase of turn 1 with priority, their hand the named cards and then lands to
/// seven, the second player's hand five lands and two fillers, and both libraries fillers enough for a few turns.
fn game_with(hand: &[&str]) -> RulesGame {
    game_with_permanents(hand, &[])
}

/// The same, with the named permanents on the first player's battlefield.
fn game_with_permanents(hand: &[&str], permanents: &[&str]) -> RulesGame {
    let cards = table();
    let (land, filler) = (cards.find("Extra Land Probe Land").unwrap(), cards.find("Extra Land Probe Filler").unwrap());
    let mut mine: Vec<CardId> = hand.iter().map(|name| cards.find(name).unwrap()).collect();
    mine.resize(7, land);
    mine.resize(20, filler);
    let mut theirs = vec![land; 5];
    theirs.resize(20, filler);
    let mut game = RulesGame::new(cards, &Setup { decks: vec![mine, theirs], seed: 1, ..Setup::default() });
    game.verify = true;
    game.state.update_turn(|t| {
        t.number = 1;
        t.active = ME;
        t.priority = ME;
        t.set_step(Step::PrecombatMain);
        t.began = true;
    });
    for name in permanents {
        put(&mut game, name, ME);
    }
    game.state.frame_push(Frame::new(FrameKind::Priority));
    game.run();
    game
}

fn put(game: &mut RulesGame, name: &str, player: PlayerId) -> ObjectId {
    let card = game.cards.find(name).unwrap();
    let id = game.state.push_obj(Obj::new(card, player, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, id);
    id
}

fn options(game: &RulesGame, player: PlayerId) -> Vec<ActionOption> {
    let request = game.pending().expect("a question");
    assert_eq!(request.player, player, "{player:?} has priority");
    match &request.kind {
        RequestKind::Priority { options } => options.clone(),
        other => panic!("expected priority, got {other:?}"),
    }
}

/// The land plays `player` is offered now.
fn land_plays(game: &RulesGame, player: PlayerId) -> Vec<ActionOption> {
    options(game, player).into_iter().filter(|o| o.kind == ActionKind::PlayLand).collect()
}

fn lands_in_hand(game: &RulesGame, player: PlayerId) -> usize {
    let land = game.cards.find("Extra Land Probe Land").unwrap();
    game.state.zone(Zone::hand(player)).iter().filter(|&&id| game.state.obj(id).card == land).count()
}

/// Plays one of `player`'s lands: a land play must be offered.
fn play_land(game: &mut RulesGame, player: PlayerId) {
    let play = land_plays(game, player).into_iter().next().expect("a land play is offered");
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(&play))).unwrap();
    game.run();
}

/// Plays lands until none is offered, with lands still in hand: how many were played.
fn play_lands_until_refused(game: &mut RulesGame, player: PlayerId) -> u32 {
    let mut played = 0;
    while !land_plays(game, player).is_empty() {
        play_land(game, player);
        played += 1;
    }
    assert!(lands_in_hand(game, player) > 0, "refused with a land in hand, not for want of one");
    played
}

/// `player` casts the named card from their hand, and both pass until it has resolved.
fn cast(game: &mut RulesGame, player: PlayerId, name: &str) {
    let card = game.cards.find(name).unwrap();
    let id = game.state.zone(Zone::hand(player)).iter().copied().find(|&id| game.state.obj(id).card == card).expect("in hand");
    let option = options(game, player).into_iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(id)).expect("castable");
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(&option))).unwrap();
    for _ in 0..16 {
        game.run();
        let request = game.pending().cloned().expect("a question");
        let resolved = game.state.zone(Zone::graveyard(player)).iter().any(|&g| game.state.obj(g).card == card);
        if resolved && game.state.stack.is_empty() && matches!(request.kind, RequestKind::Priority { .. }) {
            return;
        }
        let answer = match request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { options, .. } => {
                let confirm = options.iter().find(|o| o.kind == ActionKind::ConfirmCasting).expect("nothing to pay");
                Answer::Act(Action::of(confirm))
            }
            other => panic!("unexpected request: {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("{name} did not resolve");
}

/// Passes, declaring no attackers or blockers and discarding the first cards, until `player` has priority in the
/// precombat main phase of turn `turn`.
fn advance_to(game: &mut RulesGame, turn: u32, player: PlayerId) {
    for _ in 0..400 {
        game.run();
        let request = game.pending().cloned().expect("a question");
        if game.state.turn.number == turn && game.state.turn.step() == Step::PrecombatMain && request.player == player {
            if let RequestKind::Priority { .. } = request.kind {
                return;
            }
        }
        let answer = match &request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand[..*count as usize].to_vec()),
            other => panic!("unexpected request: {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("turn {turn} never came");
}

fn extra_land_effects(game: &RulesGame) -> Vec<(PlayerId, u32)> {
    game.state
        .effects
        .iter()
        .filter_map(|e| match e.kind {
            EffectKind::ExtraLandPlays(n) => Some((e.controller, n)),
            _ => None,
        })
        .collect()
}

/// CR 305.2, 305.2a/b: one effect lets a second land be played, not a third; and it is a permission, so nothing is asked
/// as it resolves.
#[test]
fn an_additional_land_play_allows_a_second_land_and_not_a_third() {
    let mut game = game_with(&["Extra Land Probe"]);
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(extra_land_effects(&game), vec![(ME, 1)], "the controller's, about no object, outliving the spell");
    assert_eq!(play_lands_until_refused(&mut game, ME), 2);
    assert_eq!(game.state.player(ME).lands_played, 2);
}

/// Without it, one: the control for the test above.
#[test]
fn without_it_one_land_is_played() {
    let mut game = game_with(&[]);
    assert_eq!(play_lands_until_refused(&mut game, ME), 1);
}

/// CR 305.2a: a land played before the effect began counts against the new total, which still leaves the additional one.
#[test]
fn a_land_played_before_the_effect_leaves_the_additional_one() {
    let mut game = game_with(&["Extra Land Probe"]);
    play_land(&mut game, ME);
    assert!(land_plays(&game, ME).is_empty(), "the turn's one land play is used");
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(play_lands_until_refused(&mut game, ME), 1);
    assert_eq!(game.state.player(ME).lands_played, 2);
}

/// CR 305.2: two such effects add up, and "two additional lands" is two.
#[test]
fn effects_add_up() {
    let mut game = game_with(&["Extra Land Probe", "Extra Land Probe"]);
    cast(&mut game, ME, "Extra Land Probe");
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(play_lands_until_refused(&mut game, ME), 3);

    let mut game = game_with(&["Two Extra Lands Probe"]);
    cast(&mut game, ME, "Two Extra Lands Probe");
    assert_eq!(play_lands_until_refused(&mut game, ME), 3);
}

/// "X additional lands" for an X of 0 makes no effect and changes nothing.
#[test]
fn zero_additional_lands_is_no_effect() {
    let mut game = game_with(&["No Extra Land Probe"]);
    cast(&mut game, ME, "No Extra Land Probe");
    assert!(extra_land_effects(&game).is_empty());
    assert_eq!(play_lands_until_refused(&mut game, ME), 1);
}

/// CR 305.2: the effect adds to a static ability that does the same (Icetill Explorer's), each counted once.
#[test]
fn the_effect_adds_to_a_static_ability() {
    let mut game = game_with_permanents(&[], &["Extra Land Static Probe"]);
    assert_eq!(play_lands_until_refused(&mut game, ME), 2, "the static ability alone");

    let mut game = game_with_permanents(&["Extra Land Probe"], &["Extra Land Static Probe"]);
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(play_lands_until_refused(&mut game, ME), 3);
}

/// CR 514.2: the effect ends with the turn; on the player's next turn they play one land again, and their opponent's
/// turn between was never theirs to play lands in (CR 305.3) nor more for them.
#[test]
fn the_effect_ends_with_the_turn() {
    let mut game = game_with(&["Extra Land Probe"]);
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(play_lands_until_refused(&mut game, ME), 2);
    advance_to(&mut game, 2, THEM);
    assert!(extra_land_effects(&game).is_empty(), "ended in the cleanup step");
    assert_eq!(play_lands_until_refused(&mut game, THEM), 1, "the opponent's own one");
    advance_to(&mut game, 3, ME);
    assert_eq!(play_lands_until_refused(&mut game, ME), 1, "back to one");
}

/// CR 305.2, 109.5: the effect is its controller's, not the active player's: cast on the opponent's turn, it gives the
/// opponent nothing, and its controller can't play a land on a turn not theirs (CR 305.3).
#[test]
fn the_effect_is_its_controllers_on_any_turn() {
    let mut game = game_with(&["Extra Land Probe"]);
    advance_to(&mut game, 2, THEM);
    // The opponent passes, so the probe's controller has priority on the opponent's turn.
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Pass).unwrap();
    game.run();
    assert_eq!(game.state.turn.active, THEM);
    cast(&mut game, ME, "Extra Land Probe");
    assert_eq!(extra_land_effects(&game), vec![(ME, 1)]);
    assert_eq!(play_lands_until_refused(&mut game, THEM), 1, "the opponent still has one land play");
    // The opponent passes again, and the probe's controller has priority in that main phase with lands in hand.
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Pass).unwrap();
    game.run();
    assert!(lands_in_hand(&game, ME) > 0);
    assert!(land_plays(&game, ME).is_empty(), "not on another player's turn");
}

/// The game-log tree: going back before the cast takes the effect and the land plays away, and going forward again brings
/// them back.
#[test]
fn the_effect_is_undone_and_redone_through_the_tree() {
    let mut game = game_with(&["Extra Land Probe"]);
    let before = game.checkpoint();
    let state = game.state.clone();
    cast(&mut game, ME, "Extra Land Probe");
    play_land(&mut game, ME);
    play_land(&mut game, ME);
    let after = game.checkpoint();
    let played = game.state.clone();
    game.goto(before);
    assert!(game.state == state && extra_land_effects(&game).is_empty());
    game.goto(after);
    assert!(game.state == played && extra_land_effects(&game) == vec![(ME, 1)]);
    assert!(land_plays(&game, ME).is_empty());
}
