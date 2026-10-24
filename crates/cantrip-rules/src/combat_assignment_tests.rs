//! Rules that change the quantity assigned in combat, without changing power (CR 510.1a, 609.4).

use cantrip_api::*;
use crate::{Cx, Game, static_rules};

const ME: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

fn itself(_: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    asker.source == creature
}

fn every_creature(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

static NEGATIVE: CardDef = CardDef {
    name: "Assignment Test Negative",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(-2),
    toughness: Some(4),
    ..CardDef::BLANK
};
static ABSOLUTE: CardDef = CardDef {
    name: "Assignment Test Absolute",
    damage_by_absolute_power: &[itself],
    ..NEGATIVE
};
static TOUGHNESS: CardDef = CardDef {
    name: "Assignment Test Toughness",
    types: TypeSet::of(&[CardType::Enchantment]),
    damage_by_toughness: &[every_creature],
    ..CardDef::BLANK
};

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

fn table(cards: &[(&'static CardDef, PlayerId)]) -> (Game, Vec<ObjectId>) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&NEGATIVE, &ABSOLUTE, &TOUGHNESS])));
    let mut game = Game::new(table, &crate::Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    let ids = cards.iter().map(|&(def, owner)| {
        let id = game.state.push_obj(Obj::new(table.id_of(def), owner, Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        id
    }).collect();
    (game, ids)
}

/// CR 510.1a, 609.4, 107.1b: assignment alone reads negative power as positive; zero and positive power are unchanged.
/// Even the minimum signed power has an exact unsigned magnitude, and an ordinary creature keeps assigning zero.
#[test]
fn absolute_power_assignment_preserves_actual_power_across_the_signed_range() {
    for power in [i16::MIN, -6, -2, 0, 3, i16::MAX] {
        let (mut game, ids) = table(&[(&ABSOLUTE, ME), (&NEGATIVE, ME)]);
        let card = game.cards.id_of(&ABSOLUTE);
        game.state.update_obj(ids[0], |o| o.copy = Some(Copiable::of(card).sized(power, 4)));
        let cx = cx(&mut game);
        assert_eq!(cx.chars(ids[0]).power, Some(power));
        assert_eq!(static_rules::combat_damage_of(&cx, ids[0]), power.unsigned_abs());
        assert_eq!(static_rules::combat_damage_of(&cx, ids[1]), 0);
    }
}

/// CR 707.2, 613.1f, 702.26b: copied static abilities apply, lost abilities do not, and phased-out sources are absent.
#[test]
fn absolute_power_assignment_follows_copies_ability_loss_and_phasing() {
    let (mut game, ids) = table(&[(&NEGATIVE, ME)]);
    let creature = ids[0];
    let card = game.cards.id_of(&ABSOLUTE);
    let mut cx = cx(&mut game);
    cantrip_api::Game::set_copy(&mut cx, creature, Copiable::of(card));
    assert_eq!(static_rules::combat_damage_of(&cx, creature), 2);
    cantrip_api::Game::add_effect(&mut cx, creature, creature, EffectKind::LoseAllAbilities, Duration::EndOfTurn);
    assert_eq!(static_rules::combat_damage_of(&cx, creature), 0);
    cx.st().effects_retain(|_| false);
    crate::phasing::phase_out(&mut cx, &[creature], None, false);
    assert_eq!(static_rules::combat_damage_of(&cx, creature), 0);
    crate::phasing::untap_step(&mut cx, ME);
    assert_eq!(static_rules::combat_damage_of(&cx, creature), 2);
    assert_eq!(cx.chars(creature).power, Some(-2));
}

/// CR 609.4: the power exception has no effect when another rule assigns damage using toughness. Negative toughness
/// still assigns zero rather than being made positive by a rule about power.
#[test]
fn assignment_by_toughness_ignores_the_exception_about_power() {
    let (mut game, ids) = table(&[(&ABSOLUTE, ME), (&TOUGHNESS, ME)]);
    let card = game.cards.id_of(&ABSOLUTE);
    assert_eq!(static_rules::combat_damage_of(&cx(&mut game), ids[0]), 4);
    game.state.update_obj(ids[0], |o| o.copy = Some(Copiable::of(card).sized(-2, -3)));
    assert_eq!(static_rules::combat_damage_of(&cx(&mut game), ids[0]), 0);
    cantrip_api::Game::add_effect(&mut cx(&mut game), ids[1], ids[1], EffectKind::LoseAllAbilities, Duration::EndOfTurn);
    assert_eq!(static_rules::combat_damage_of(&cx(&mut game), ids[0]), 2);
}

/// CR 510.1a-d: the same assignment rule applies unblocked, blocked and blocking, while an ordinary creature with
/// negative power deals no combat damage. No assignment changes the power that a later effect reads.
#[test]
fn absolute_power_is_used_for_unblocked_attackers_blocked_attackers_and_blockers() {
    for arrangement in 0..3 {
        let blocking = arrangement == 2;
        let (mut game, ids) = table(&[(&ABSOLUTE, if blocking { THEM } else { ME }), (&NEGATIVE, if blocking { ME } else { THEM })]);
        let [absolute, normal] = ids[..] else { unreachable!() };
        let attacker = if blocking { normal } else { absolute };
        let blocker = if blocking { absolute } else { normal };
        game.state.set_attackers(vec![Attacker { blocked: arrangement > 0, ..Attacker::new(attacker, Target::Player(THEM), THEM, TypeSet::NONE) }]);
        if arrangement > 0 {
            game.state.set_blocks(vec![Block { blocker, attacker, first_strike: false }]);
        }
        game.state.update_turn(|t| t.set_step(Step::CombatDamage));
        crate::turn::begin_step(&mut cx(&mut game)).expect("no assignment choice");
        assert_eq!(game.state.player(THEM).life, if arrangement == 0 { 18 } else { 20 });
        assert_eq!(game.state.obj(normal).damage, if arrangement == 0 { 0 } else { 2 });
        assert_eq!(game.state.obj(absolute).damage, 0);
        assert_eq!(cx(&mut game).chars(absolute).power, Some(-2));
    }
}

/// CR 510.1a, 704.5g: the minimum signed power assigns 32,768 damage, which is lethal to a positive-toughness blocker.
/// Marked damage must not become negative when the next state-based check compares it with toughness.
#[test]
fn minimum_signed_power_deals_32768_combat_damage_and_destroys_its_blocker() {
    let (mut game, ids) = table(&[(&ABSOLUTE, ME), (&NEGATIVE, THEM)]);
    let [attacker, blocker] = ids[..] else { unreachable!() };
    let card = game.cards.id_of(&ABSOLUTE);
    game.state.update_obj(attacker, |o| o.copy = Some(Copiable::of(card).sized(i16::MIN, 4)));
    game.state.set_attackers(vec![Attacker { blocked: true, ..Attacker::new(attacker, Target::Player(THEM), THEM, TypeSet::NONE) }]);
    game.state.set_blocks(vec![Block { blocker, attacker, first_strike: false }]);
    game.state.update_turn(|t| t.set_step(Step::CombatDamage));
    crate::turn::begin_step(&mut cx(&mut game)).expect("no assignment choice");
    assert_eq!(game.state.obj(blocker).damage, 32768);
    assert_eq!(cx(&mut game).chars(attacker).power, Some(i16::MIN));
    crate::settle::state_based_actions_alone(&mut cx(&mut game)).expect("no choices");
    assert!(!crate::live(&game.state, blocker));
    assert!(crate::live(&game.state, attacker));
    let graveyard = game.state.zone(Zone::graveyard(THEM));
    assert_eq!(graveyard.len(), 1);
    assert_eq!(game.state.obj(graveyard[0]).card, game.cards.id_of(&NEGATIVE));
}

/// CR 702.19b: an indestructible blocker already marked with at least 32,768 damage needs zero more assigned as lethal.
/// The assignment prompt and actual trample damage use that exact subtraction up to u16::MAX, even with deathtouch;
/// with no marked damage, deathtouch needs one instead.
#[test]
fn trample_accounts_for_large_marked_damage_and_deathtouch_without_signed_narrowing() {
    for (marked, deathtouch, to_blocker) in [(32768, false, 0), (u16::MAX, false, 0), (32768, true, 0), (u16::MAX, true, 0), (0, true, 1)] {
        let (mut game, ids) = table(&[(&ABSOLUTE, ME), (&NEGATIVE, THEM)]);
        let [attacker, blocker] = ids[..] else { unreachable!() };
        cantrip_api::Game::add_effect(&mut cx(&mut game), attacker, attacker, EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample])), Duration::EndOfTurn);
        if deathtouch {
            cantrip_api::Game::add_effect(&mut cx(&mut game), attacker, attacker, EffectKind::AddKeywords(Keywords::of(&[Keyword::Deathtouch])), Duration::EndOfTurn);
        }
        cantrip_api::Game::add_effect(&mut cx(&mut game), blocker, blocker, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
        game.state.update_obj(blocker, |o| o.damage = marked);
        game.state.set_attackers(vec![Attacker { blocked: true, ..Attacker::new(attacker, Target::Player(THEM), THEM, TypeSet::NONE) }]);
        game.state.set_blocks(vec![Block { blocker, attacker, first_strike: false }]);
        game.state.update_turn(|t| t.set_step(Step::CombatDamage));
        let frame = game.state.agenda.len();
        game.state.frame_push(cantrip_state::Frame::new(cantrip_state::FrameKind::BeginStep));
        let mut cx = cx(&mut game);
        cx.frame = frame;
        assert!(matches!(crate::turn::begin_step(&mut cx), Err(crate::Stop::Suspend)));
        let request = cx.s().agenda[frame].pending.clone().expect("a combat assignment");
        let RequestKind::AssignCombatDamage { divisions } = &request.kind else { panic!("combat assignment expected") };
        assert_eq!(divisions.len(), 1);
        assert_eq!(divisions[0].lethal, [to_blocker]);
        assert_eq!(divisions[0].suggested, [to_blocker]);
        assert_eq!(divisions[0].suggested_to_defender, 2 - to_blocker);
        let answer = Answer::Shares(DamageDivision::suggested_shares(divisions));
        crate::check_answer(cx.g, &request, &answer).expect("all damage past an already lethally marked blocker is legal");
        cx.st().update_frame(frame, |f| {
            f.pending = None;
            f.answers.push((None, answer));
        });
        crate::turn::begin_step(&mut cx).expect("the assignment was answered");
        assert_eq!(cx.s().player(THEM).life, 18 + i32::from(to_blocker));
        assert_eq!(cx.s().obj(blocker).damage, marked + to_blocker);
        crate::settle::state_based_actions_alone(&mut cx).expect("no choices");
        assert!(crate::live(cx.s(), blocker));
    }
}
