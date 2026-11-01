//! Prevention effects a spell or ability made that do more than prevent (CR 615.5), and "the next time ... would deal
//! damage, prevent that damage" (CR 615.8), over test cards: the rest of the effect happens once the damage event has been
//! dealt and is told what was prevented; the "next time" is used up by the one event; neither applies to damage that can't
//! be prevented (CR 615.12); both are ordered with the other effects on the damage (CR 616.1); and the source chosen is
//! asked of as it last existed (CR 608.2h, 609.7a).

use crate::ops::{damage, damage_together, exile_all};
use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_mechanics::prevention;

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

static BEAR: CardDef = CardDef {
    name: "Prevention Then Test Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

/// "[The effect's source] deals that much damage to that source's controller" (Deflecting Palm): the chosen source's controller
/// as it last existed (CR 608.2h).
fn deflect(g: &mut dyn cantrip_api::Game, asker: &Asker, chosen: ObjectId, prevented: &[Prevented]) {
    let to = g.controller(chosen);
    let amount = u16::try_from(Prevented::total(prevented)).expect("test damage is small");
    g.damage(asker.source, Target::Player(to), amount);
}

/// "[The effect's source] deals that much damage to [the other player]" (Refraction Trap's "any target", as a fixed one).
fn to_them(g: &mut dyn cantrip_api::Game, asker: &Asker, _: ObjectId, prevented: &[Prevented]) {
    let amount = u16::try_from(Prevented::total(prevented)).expect("test damage is small");
    g.damage(asker.source, Target::Player(THEM), amount);
}

/// The parts it was told of, as life its controller gains: one for each part, so a test can count them.
fn count_parts(g: &mut dyn cantrip_api::Game, asker: &Asker, _: ObjectId, prevented: &[Prevented]) {
    g.gain_life(asker.controller, prevented.len() as i32);
}

fn by_chosen(_: &dyn Query, _: &Asker, chosen: ObjectId, d: &DamageEvent) -> bool {
    d.source == chosen
}

/// Damage from the source the effect is about.
const FROM_CHOSEN: GrantedPreventionDef =
    GrantedPreventionDef { hint: "prevents the damage from the chosen source", damage: "damage", of_it: None, applies: by_chosen, then: None };

static PALM: CardDef = CardDef {
    name: "Prevention Then Test Palm",
    types: TypeSet::of(&[CardType::Instant]),
    granted_preventions: &[
        GrantedPreventionDef { then: Some(deflect), ..FROM_CHOSEN },
        GrantedPreventionDef { then: Some(to_them), ..prevention::ANY_DAMAGE },
        prevention::ANY_DAMAGE,
        GrantedPreventionDef { then: Some(count_parts), ..FROM_CHOSEN },
    ],
    ..CardDef::BLANK
};

/// Its definitions, by index in `granted_preventions`.
const DEFLECT: u8 = 0;
const REFRACT: u8 = 1;
const PLAIN: u8 = 2;
const COUNT: u8 = 3;

struct Table {
    game: Game,
    /// The resolved spell whose effects these are, in its owner's graveyard.
    palm: ObjectId,
    /// A creature THEM controls, the source chosen.
    bear: ObjectId,
    /// Another creature THEM controls, a source no effect watches.
    other: ObjectId,
    /// A creature YOU control.
    mine: ObjectId,
}

fn table() -> Table {
    let cards: &'static CardTable = Box::leak(Box::new(CardTable::new([&BEAR, &PALM])));
    let mut game = Game::new(cards, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    // Turn 1: no turn of the game is one in which damage can't be prevented (`no_prevention_turn` starts at 0).
    game.state.update_turn(|t| t.number = 1);
    let put = |game: &mut Game, card: &'static CardDef, owner: PlayerId, zone: Zone| {
        let id = game.state.push_obj(Obj::new(cards.id_of(card), owner, zone));
        game.state.zone_push(zone, id);
        id
    };
    let palm = put(&mut game, &PALM, YOU, Zone::graveyard(YOU));
    let bear = put(&mut game, &BEAR, THEM, Zone::BATTLEFIELD);
    let other = put(&mut game, &BEAR, THEM, Zone::BATTLEFIELD);
    let mine = put(&mut game, &BEAR, YOU, Zone::BATTLEFIELD);
    Table { game, palm, bear, other, mine }
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

/// A frame to ask questions in, its answers given in advance: what the run reads back as it asks (`Cx::ask_core`).
fn answered(cx: &mut Cx, answers: &[Answer]) {
    let frame = cx.s().agenda.len();
    cx.st().frame_push(cantrip_state::Frame::new(cantrip_state::FrameKind::BeginStep));
    for answer in answers {
        cx.st().update_frame(frame, |f| f.answers.push((None, answer.clone())));
    }
    cx.frame = frame;
}

/// The effect of the Palm's `index`th definition about `affected`, of damage to `to`, until end of turn.
fn shield(cx: &mut Cx, palm: ObjectId, affected: ObjectId, index: u8, to: Target, shield: Shield) {
    let card = cx.s().obj(palm).card;
    cantrip_api::Game::add_effect(cx, palm, affected, EffectKind::Prevention { card, index, to, shield }, Duration::EndOfTurn);
}

fn prevention_effects(cx: &Cx) -> Vec<Shield> {
    cx.s().effects.iter().filter_map(|e| match e.kind {
        EffectKind::Prevention { shield, .. } => Some(shield),
        _ => None,
    }).collect()
}

/// The damage events in the history, in order: source, recipient, amount.
fn dealt(cx: &Cx) -> Vec<(ObjectId, Target, u16)> {
    cx.g.log.iter().filter_map(|r| match r.event {
        Event::Damage { source, target, amount, .. } => Some((source, target, amount)),
        _ => None,
    }).collect()
}

fn prevented(cx: &Cx) -> Vec<(ObjectId, Target, u16)> {
    cx.g.log.iter().filter_map(|r| match r.event {
        Event::DamagePrevented { source, target, amount, .. } => Some((source, target, amount)),
        _ => None,
    }).collect()
}

/// CR 615.8, 615.5: the next time the chosen source would deal damage to you, all of it is prevented and the effect is used
/// up; then the Palm deals that much damage to the source's controller. The next damage from it is dealt normally, and
/// damage from a source the effect does not watch was never its to prevent.
#[test]
fn the_next_time_is_used_up_by_one_event_and_its_rest_deals_that_much() {
    let Table { mut game, palm, bear, other, .. } = table();
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, bear, DEFLECT, Target::Player(YOU), Shield::NextTime);
    assert_eq!(damage(&mut cx, other, Target::Player(YOU), 1, false), 1, "another source's damage is dealt");
    assert_eq!(prevention_effects(&cx), [Shield::NextTime], "and does not use the effect up");
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, true), 0);
    assert_eq!(cx.s().player(YOU).life, 19);
    assert_eq!(cx.s().player(THEM).life, 17, "the Palm deals the 3 prevented to the source's controller");
    assert_eq!(prevented(&cx), [(bear, Target::Player(YOU), 3)]);
    assert_eq!(dealt(&cx), [(other, Target::Player(YOU), 1), (palm, Target::Player(THEM), 3)], "dealt by the Palm, after the prevention");
    assert!(prevention_effects(&cx).is_empty(), "used up");
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 2, false), 2, "the next damage from it is dealt normally");
    assert_eq!(cx.s().player(YOU).life, 17);
    assert_eq!(cx.s().player(THEM).life, 17);
}

/// CR 615.5: the rest of a shield's effect is told how much it prevented, not how much would have been dealt, and happens
/// after the damage event - the 1 the shield left is dealt first.
#[test]
fn the_rest_of_a_shield_gets_the_amount_prevented_after_the_event() {
    let Table { mut game, palm, bear, .. } = table();
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, palm, REFRACT, Target::Player(YOU), Shield::Next(2));
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, false), 1);
    assert_eq!(cx.s().player(YOU).life, 19);
    assert_eq!(cx.s().player(THEM).life, 18, "2 prevented, 2 dealt to them");
    assert_eq!(dealt(&cx), [(bear, Target::Player(YOU), 1), (palm, Target::Player(THEM), 2)]);
    assert!(prevention_effects(&cx).is_empty(), "the shield is used up");
}

/// CR 510.2, 615.5: combat damage is one event too - the rest of a shield's effect runs once it has been dealt, in the
/// combat damage step, before anything else happens.
#[test]
fn the_rest_of_a_shield_runs_after_combat_damage() {
    let Table { mut game, palm, bear, .. } = table();
    game.state.update_turn(|t| t.active = THEM);
    game.state.set_attackers(vec![Attacker::new(bear, Target::Player(YOU), YOU, TypeSet::NONE)]);
    game.state.update_turn(|t| t.set_step(Step::CombatDamage));
    let frame = game.state.agenda.len();
    game.state.frame_push(cantrip_state::Frame::new(cantrip_state::FrameKind::BeginStep));
    let mut cx = cx(&mut game);
    cx.frame = frame;
    shield(&mut cx, palm, palm, REFRACT, Target::Player(YOU), Shield::Next(2));
    crate::turn::begin_step(&mut cx).expect("no questions");
    assert_eq!(cx.s().player(YOU).life, 19, "the 1 the shield left");
    assert_eq!(cx.s().player(THEM).life, 18, "the 2 prevented, dealt to them");
    assert_eq!(dealt(&cx), [(bear, Target::Player(YOU), 1), (palm, Target::Player(THEM), 2)]);
}

/// CR 615.8, 120.2: "the next time [the source] would deal damage" (of damage to anyone) is one damage event however many
/// recipients it has: every part of it is prevented, the rest of the effect runs once and is told of each part, and the
/// effect is used up after it - not after the first part.
#[test]
fn the_next_time_prevents_every_part_of_one_simultaneous_event() {
    let Table { mut game, palm, bear, mine, .. } = table();
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, bear, COUNT, Target::None, Shield::NextTime);
    let hits = [(bear, Target::Player(YOU), 2), (bear, Target::Object(mine), 2)];
    assert_eq!(damage_together(&mut cx, &hits), [0, 0]);
    assert_eq!(cx.s().obj(mine).damage, 0);
    assert_eq!(cx.s().player(YOU).life, 22, "told of both parts, once");
    assert!(prevention_effects(&cx).is_empty());
    assert_eq!(damage_together(&mut cx, &hits), [2, 2], "the next event is dealt");
}

/// CR 615.12: damage that can't be prevented isn't, the rest of the effect ("if damage is prevented this way") does
/// nothing, and the effect is not used up by it (CR 609.7b).
#[test]
fn damage_that_cant_be_prevented_leaves_the_effect_and_does_nothing_more() {
    let Table { mut game, palm, bear, .. } = table();
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, bear, DEFLECT, Target::Player(YOU), Shield::NextTime);
    cantrip_api::Game::damage_cant_be_prevented(&mut cx);
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, false), 3);
    assert_eq!(cx.s().player(YOU).life, 17);
    assert_eq!(cx.s().player(THEM).life, 20);
    assert!(prevented(&cx).is_empty());
    assert_eq!(prevention_effects(&cx), [Shield::NextTime]);
}

/// CR 616.1: a "next time" with more to do is ordered with a shield on the same damage by the player dealt it, and the
/// order changes what happens. The shield first prevents 2 and the Palm the 1 left, which it deals; the Palm first prevents
/// all 3, which it deals, and the shield is untouched.
#[test]
fn the_player_dealt_the_damage_orders_it_with_a_shield() {
    for (first, them, left) in [(1, 19, Vec::new()), (0, 17, vec![Shield::Next(2)])] {
        let Table { mut game, palm, bear, .. } = table();
        let mut cx = cx(&mut game);
        shield(&mut cx, palm, bear, DEFLECT, Target::Player(YOU), Shield::NextTime);
        shield(&mut cx, palm, palm, PLAIN, Target::Player(YOU), Shield::Next(2));
        answered(&mut cx, &[Answer::Index(first)]);
        assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, false), 0);
        assert_eq!(cx.cursor, 1, "the order was asked");
        assert_eq!(cx.s().player(YOU).life, 20);
        assert_eq!(cx.s().player(THEM).life, them);
        assert_eq!(prevention_effects(&cx), left);
    }
}

/// CR 616.1f, 609.7b: another effect applied first prevents all the damage; what is left is no damage the "next time"
/// applies to (0 damage is no event), so it prevents nothing, does nothing more, and is not used up.
#[test]
fn damage_another_effect_prevented_in_full_leaves_the_next_time_waiting() {
    let Table { mut game, palm, bear, .. } = table();
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, bear, DEFLECT, Target::Player(YOU), Shield::NextTime);
    shield(&mut cx, palm, palm, PLAIN, Target::Player(YOU), Shield::All);
    answered(&mut cx, &[Answer::Index(1)]);
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, false), 0);
    assert_eq!(cx.cursor, 1, "the order was asked: which applies first matters");
    assert_eq!(cx.s().player(THEM).life, 20);
    assert_eq!(prevented(&cx).len(), 1);
    assert_eq!(prevention_effects(&cx), [Shield::NextTime, Shield::All]);
}

/// CR 608.2h: a source that has left the battlefield deals its damage as it last existed (a dies trigger's), and "that
/// source's controller" is its controller then - not its owner, whose exile it is in now.
#[test]
fn a_source_that_has_left_is_asked_of_as_it_last_existed() {
    let Table { mut game, palm, bear, .. } = table();
    game.state.update_obj(bear, |o| o.owner = YOU);
    let mut cx = cx(&mut game);
    shield(&mut cx, palm, bear, DEFLECT, Target::Player(YOU), Shield::NextTime);
    assert_eq!(exile_all(&mut cx, &[bear]).len(), 1);
    assert_eq!(damage(&mut cx, bear, Target::Player(YOU), 3, false), 0);
    assert_eq!(cx.s().player(YOU).life, 20);
    assert_eq!(cx.s().player(THEM).life, 17, "its last controller is dealt the damage");
}
