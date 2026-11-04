//! One instruction's life losses (CR 101.4, 119.3, 603.2c, 608.2h), over test-only definitions of printed abilities.
//! Gul Dukat and Archfiend of Despair supply different amounts; Exsanguinate supplies one amount and uses the total.

use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);
const FIRST: PlayerId = PlayerId(1);
const ACTIVE: PlayerId = PlayerId(2);
const LAST: PlayerId = PlayerId(3);

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

fn opponent_lost_life(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match *event {
        Event::LifeChange { player, delta } if player != me.controller && delta < 0 => Fired::about(Target::Player(player)),
        _ => None,
    }
}

fn opponent_lost_one(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match *event {
        Event::LifeChange { delta: -1, .. } => opponent_lost_life(q, me, event),
        _ => None,
    }
}

static WATCHER: CardDef = CardDef {
    name: "Life Together Watcher",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[
        triggered(when_asked(EventMask::LIFE_CHANGE, opponent_lost_one), nothing,
            "Whenever one or more opponents each lose exactly 1 life, put a +1/+1 counter on Ob Nixilis. Exile the top card of your library. Until your next end step, you may play that card.").once_per_batch(),
        triggered(when_asked(EventMask::LIFE_CHANGE, opponent_lost_life), nothing,
            "Whenever an opponent loses life, that player mills that many cards."),
    ],
    ..CardDef::BLANK
};

/// Bloodletter of Aclazotz's replacement, with assertions that all replacements see the original instruction's
/// pre-loss state and history, even for a player later in APNAP order.
fn opponent_loses_on_your_turn(q: &dyn Query, me: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::LifeChange(l) if l.amount < 0 && l.player != me.controller && q.state().turn.active == me.controller)
}

fn twice(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    assert!(q.state().players.iter().all(|p| p.life == 20), "no original life update before all replacements");
    assert!(!q.history().records.iter().any(|r| matches!(r.event, Event::LifeChange { .. })), "no original event before all replacements");
    if let Proposal::LifeChange(l) = proposal {
        l.amount *= 2;
    }
}

static DOUBLER: CardDef = CardDef {
    name: "Life Together Doubler",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::LifeChange, opponent_loses_on_your_turn, twice,
        "If an opponent would lose life during your turn, they lose twice that much life instead.")],
    ..CardDef::BLANK
};

fn own_total(_: &dyn Query, me: &Asker, player: PlayerId) -> bool {
    player == me.controller
}

/// Platinum Emperion's prohibition.
static EMPERION: CardDef = CardDef {
    name: "Life Together Fixed Total",
    types: TypeSet::of(&[CardType::Enchantment]),
    cant_gain_life: &[own_total],
    cant_lose_life: &[own_total],
    ..CardDef::BLANK
};

fn opponent_gains(_: &dyn Query, me: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::LifeChange(l) if l.amount > 0 && l.player != me.controller)
}

fn gain_to_loss(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::LifeChange(l) = proposal {
        l.amount = -l.amount;
    }
}

/// Tainted Remedy's real sign-changing replacement, for the single-player pipeline regression.
static REMEDY: CardDef = CardDef {
    name: "Life Together Remedy",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::LifeChange, opponent_gains, gain_to_loss,
        "If an opponent would gain life, that player loses that much life instead.")],
    ..CardDef::BLANK
};

/// Exsanguinate at X=1: the return value is used by the next instruction, after every opponent's loss.
fn drain(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let lost = g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, lost);
    Ok(())
}

static DRAIN: CardDef = CardDef {
    name: "Life Together Drain",
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[], drain),
    ..CardDef::BLANK
};

fn game() -> Game {
    let table = Box::leak(Box::new(CardTable::new([&WATCHER, &DOUBLER, &EMPERION, &REMEDY, &DRAIN])));
    let mut g = Game::new(table, &Setup { decks: vec![Vec::new(); 4], ..Default::default() });
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    for player in [YOU, FIRST, ACTIVE, LAST] {
        g.state.update_player(player, |p| p.drew_from_empty = false);
    }
    g.state.update_turn(|t| {
        t.active = ACTIVE;
        t.priority = ACTIVE;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g
}

fn put(g: &mut Game, def: &'static CardDef, controller: PlayerId) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), controller, Zone::BATTLEFIELD));
    let timestamp = g.state.timestamp();
    g.state.update_obj(id, |o| o.timestamp = timestamp);
    g.state.zone_push(Zone::BATTLEFIELD, id);
    g.state.see_statics();
    id
}

fn cx(g: &mut Game) -> Cx<'_> {
    Cx { g, frame: 0, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

fn changes(g: &Game) -> Vec<(PlayerId, i32)> {
    g.log.iter().filter_map(|r| match r.event {
        Event::LifeChange { player, delta } => Some((player, delta)),
        _ => None,
    }).collect()
}

fn life(g: &Game) -> Vec<i32> {
    g.state.players.iter().map(|p| p.life).collect()
}

fn trigger_count(g: &Game, source: ObjectId, index: u8) -> usize {
    g.state.triggers.iter().filter(|t| t.source == source && t.ability.index == index).count()
}

#[test]
fn different_losses_are_one_batch_but_individual_occurrences() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let lost = cantrip_api::Game::lose_life_together(&mut cx(&mut g), &[(FIRST, 1), (ACTIVE, 3), (LAST, 1)]);
    assert_eq!(lost, 5);
    assert_eq!(life(&g), vec![20, 19, 17, 19]);
    assert_eq!(changes(&g), vec![(ACTIVE, -3), (LAST, -1), (FIRST, -1)]);
    assert_eq!(trigger_count(&g, watcher, 0), 1, "Ob Nixilis sees one occurrence, not two");
    assert_eq!(trigger_count(&g, watcher, 1), 3, "Mindcrank sees each opponent's loss");
}

#[test]
fn each_opponent_uses_the_same_simultaneous_operation() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, ACTIVE);
    assert_eq!(cantrip_api::Game::each_opponent_loses_life(&mut cx(&mut g), ACTIVE, 1), 3);
    assert_eq!(changes(&g), vec![(LAST, -1), (YOU, -1), (FIRST, -1)]);
    assert_eq!(trigger_count(&g, watcher, 0), 1);
    assert_eq!(trigger_count(&g, watcher, 1), 3);
}

#[test]
fn a_players_entries_are_one_loss_replaced_and_announced_once() {
    let mut g = game();
    put(&mut g, &DOUBLER, ACTIVE);
    let watcher = put(&mut g, &WATCHER, ACTIVE);
    assert_eq!(cantrip_api::Game::lose_life_together(&mut cx(&mut g), &[(FIRST, 1), (YOU, 3), (FIRST, 2)]), 12);
    assert_eq!(changes(&g), vec![(YOU, -6), (FIRST, -6)]);
    assert_eq!(life(&g), vec![14, 14, 20, 20]);
    assert_eq!(trigger_count(&g, watcher, 1), 2);
}

#[test]
fn blocked_losses_are_not_replaced_or_counted() {
    let mut g = game();
    put(&mut g, &EMPERION, FIRST);
    let source = put(&mut g, &DOUBLER, ACTIVE);
    put(&mut g, &DOUBLER, ACTIVE);
    g.state.effect_push(ContinuousEffect {
        source, affected: source, controller: LAST, timestamp: 1, words: Words::NONE,
        kind: EffectKind::LifeTotalCantChange(LAST), duration: Duration::EndOfTurn,
    });
    assert_eq!(cantrip_api::Game::lose_life_together(&mut cx(&mut g), &[(FIRST, 2), (LAST, 7)]), 0);
    assert_eq!(life(&g), vec![20; 4]);
    assert!(changes(&g).is_empty());
    assert!(g.state.agenda[0].pending.is_none(), "no replacement choices for prohibited losses");
    assert_eq!(cantrip_api::Game::lose_life_together(&mut cx(&mut g), &[(FIRST, 2), (ACTIVE, 4), (LAST, 7)]), 4);
    assert_eq!(changes(&g), vec![(ACTIVE, -4)]);
}

#[test]
fn nonpositive_amounts_and_departed_players_do_nothing() {
    let mut g = game();
    g.state.update_player(FIRST, |p| p.lost = true);
    put(&mut g, &DOUBLER, ACTIVE);
    put(&mut g, &DOUBLER, ACTIVE);
    let mut c = cx(&mut g);
    assert_eq!(cantrip_api::Game::lose_life_together(&mut c, &[]), 0);
    assert_eq!(cantrip_api::Game::lose_life_together(&mut c, &[(FIRST, 8), (YOU, 0), (LAST, -2)]), 0);
    assert_eq!(cantrip_api::Game::each_opponent_loses_life(&mut c, ACTIVE, 0), 0);
    assert_eq!(life(&g), vec![20; 4]);
    assert!(g.log.is_empty());
    assert!(g.state.agenda[0].pending.is_none());
}

#[test]
fn live_players_can_lose_below_zero_and_large_amounts_are_kept_whole() {
    let mut g = game();
    g.state.update_player(FIRST, |p| p.life = 0);
    assert_eq!(cantrip_api::Game::lose_life_together(&mut cx(&mut g), &[(FIRST, 70_000), (LAST, 80_000)]), 150_000);
    assert_eq!(life(&g), vec![20, -70_000, 20, -79_980]);
    assert!(g.state.in_game(FIRST), "losing life does not run state-based actions mid-instruction");
}

#[test]
fn single_player_changes_remain_individual_and_keep_sign_changing_replacements() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    cantrip_api::Game::lose_life(&mut cx(&mut g), FIRST, 1);
    cantrip_api::Game::lose_life(&mut cx(&mut g), LAST, 1);
    assert_eq!(trigger_count(&g, watcher, 0), 2, "separate calls remain separate events");
    put(&mut g, &REMEDY, YOU);
    cantrip_api::Game::gain_life(&mut cx(&mut g), FIRST, 3);
    cantrip_api::Game::set_life_total(&mut cx(&mut g), LAST, 22);
    assert_eq!(life(&g), vec![20, 16, 20, 16]);
    assert_eq!(changes(&g), vec![(FIRST, -1), (LAST, -1), (FIRST, -3), (LAST, -3)]);
    put(&mut g, &EMPERION, FIRST);
    cantrip_api::Game::gain_life(&mut cx(&mut g), FIRST, 3);
    cantrip_api::Game::lose_life(&mut cx(&mut g), FIRST, 3);
    assert_eq!(g.state.player(FIRST).life, 16);
    assert_eq!(changes(&g).len(), 4);
}

#[test]
fn concession_removes_replacements_before_the_question_is_rebuilt() {
    let mut g = game();
    put(&mut g, &DOUBLER, ACTIVE);
    put(&mut g, &DOUBLER, ACTIVE);
    let card = g.cards.id_of(&DRAIN);
    let source = g.state.push_obj(Obj::new(card, YOU, Zone::STACK));
    g.state.zone_push(Zone::STACK, source);
    g.state.stack_push(crate::actions::stack_item(StackKind::Spell, source, AbilityRef::spell(card), YOU, &[]));
    let serial = g.state.stack.last().unwrap().serial;
    g.state.frame_push(Frame::new(FrameKind::Resolve { serial }));
    g.run();
    assert_eq!(g.pending().unwrap().player, LAST);
    assert!(matches!(g.pending().unwrap().kind, RequestKind::ChooseReplacement { .. }));

    g.concede(ACTIVE);
    assert!(!matches!(g.pending().map(|r| &r.kind), Some(RequestKind::ChooseReplacement { .. })), "removed Bloodletters must not be offered");
    assert_eq!(changes(&g), vec![(LAST, -1), (FIRST, -1), (YOU, 2)]);
    assert_eq!(life(&g), vec![22, 19, 20, 19]);
}

/// Real replacement questions and resolution replay, not preloaded answers: every opponent orders two Bloodletters
/// before Exsanguinate changes any total. Undo/redo restores both the pre-instruction request and the final history.
#[test]
fn replacement_choices_are_apnap_before_all_updates_and_survive_undo_redo() {
    let mut g = game();
    let one = put(&mut g, &DOUBLER, ACTIVE);
    let two = put(&mut g, &DOUBLER, ACTIVE);
    let card = g.cards.id_of(&DRAIN);
    let source = g.state.push_obj(Obj::new(card, ACTIVE, Zone::STACK));
    g.state.zone_push(Zone::STACK, source);
    g.state.stack_push(crate::actions::stack_item(StackKind::Spell, source, AbilityRef::spell(card), ACTIVE, &[]));
    let serial = g.state.stack.last().unwrap().serial;
    g.state.frame_push(Frame::new(FrameKind::Resolve { serial }));
    g.run();
    let before = g.at;
    for player in [LAST, YOU, FIRST] {
        let request = g.pending().cloned().expect("replacement question");
        assert_eq!(request.player, player, "APNAP, not seat order or the loss input order");
        let RequestKind::ChooseReplacement { life: amount, sources, .. } = &request.kind else { panic!("{:?}", request.kind) };
        assert_eq!(*amount, Some(-1));
        assert_eq!(*sources, vec![one, two]);
        assert_eq!(life(&g), vec![20; 4], "every choice precedes the original updates");
        assert!(changes(&g).is_empty());
        g.answer(request.id, player, Answer::Index(1)).expect("valid order");
        g.run();
    }
    let after = g.at;
    let expected = vec![(LAST, -4), (YOU, -4), (FIRST, -4), (ACTIVE, 12)];
    assert_eq!(changes(&g), expected);
    assert_eq!(life(&g), vec![16, 16, 32, 16]);
    g.goto(before);
    assert_eq!(g.pending().unwrap().player, LAST);
    assert_eq!(life(&g), vec![20; 4]);
    assert!(changes(&g).is_empty());
    g.goto(after);
    assert_eq!(changes(&g), expected);
    assert_eq!(life(&g), vec![16, 16, 32, 16]);
}
