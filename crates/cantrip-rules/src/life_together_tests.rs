//! One instruction's life losses and simultaneous damage results (CR 101.4, 119.3, 120.4, 603.2c, 608.2h), over
//! test-only definitions of printed abilities. Gul Dukat and Archfiend of Despair supply different amounts;
//! Exsanguinate supplies one amount and uses the total; Pyrohemia and combat share the damage-result pipeline.

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

/// Pyrohemia's activated ability, including creatures: the four-player position is outside the two-seat script harness.
fn pyrohemia(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let mut hits: Vec<_> = g.state().apnap().map(|p| (item.object, Target::Player(p), 1)).collect();
    hits.extend(g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature))
        .map(|id| (item.object, Target::Object(id), 1)));
    g.damage_together(&hits);
    Ok(())
}

static PYROHEMIA: CardDef = CardDef {
    name: "Damage Life Pyrohemia",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[ability(cost("{R}"), pyrohemia, "{R}: Pyrohemia deals 1 damage to each creature and each player.")],
    ..CardDef::BLANK
};

static LIFELINKER: CardDef = CardDef {
    name: "Damage Life Vampire Nighthawk",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2), toughness: Some(3),
    keywords: Keywords::of(&[Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink]),
    ..CardDef::BLANK
};

static HAWK: CardDef = CardDef {
    name: "Damage Life Healer's Hawk",
    power: Some(1), toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying, Keyword::Lifelink]),
    ..LIFELINKER
};

static INFECT: CardDef = CardDef {
    name: "Damage Life Blighted Agent",
    power: Some(1), toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Infect, Keyword::CantBeBlocked]),
    ..LIFELINKER
};

static TOXIC: CardDef = CardDef {
    name: "Damage Life Bilious Skulldweller",
    power: Some(1), toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Deathtouch]),
    toxic: 1,
    ..LIFELINKER
};

fn to_you(_: &dyn Query, me: &Asker, damage: &DamageEvent) -> bool {
    damage.target == Target::Player(me.controller)
}

fn one_damage(_: &dyn Query, _: &Asker, _: &DamageEvent) -> u16 {
    1
}

static ARMOR: CardDef = CardDef {
    name: "Damage Life Urza's Armor",
    types: TypeSet::of(&[CardType::Artifact]),
    preventions: &[PreventionDef { hint: "prevent 1 damage", applies: to_you, amount: Some(one_damage) }],
    ..CardDef::BLANK
};

fn gained(_: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match *event {
        Event::LifeChange { player, delta } if player == me.controller && delta > 0 => Fired::PLAIN,
        _ => None,
    }
}

static PRIDEMATE: CardDef = CardDef {
    name: "Damage Life Ajani's Pridemate",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2), toughness: Some(2),
    abilities: &[triggered(when_asked(EventMask::LIFE_CHANGE, gained), nothing,
        "Whenever you gain life, put a +1/+1 counter on Ajani's Pridemate.")],
    ..CardDef::BLANK
};

fn own_gain(_: &dyn Query, me: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::LifeChange(l) if l.player == me.controller && l.amount > 0)
}

fn plus_one(q: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    assert!(q.state().players.iter().all(|p| p.life == 20), "lifelink replacements precede every original life update");
    assert!(!q.history().records.iter().any(|r| matches!(r.event, Event::LifeChange { .. })), "no original life announcement yet");
    if let Proposal::LifeChange(l) = proposal {
        l.amount += 1;
    }
}

static ANGEL: CardDef = CardDef {
    name: "Damage Life Angel of Vitality",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::LifeChange, own_gain, plus_one,
        "If you would gain life, you gain that much life plus 1 instead.")],
    ..CardDef::BLANK
};

/// Darien's trigger sees the completed damage results, but its question about the past excludes the whole event,
/// including its life changes (CR 120.4c-d, 603.2c). Used only in the Pyrohemia position below.
fn dealt_to_you(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if matches!(event, Event::Damage { target: Target::Player(p), .. } if *p == me.controller) {
        assert!(q.state().players.iter().all(|p| p.life == 19));
        assert!(!q.history().records.iter().any(|r| matches!(r.event, Event::LifeChange { .. } | Event::Damage { .. })),
            "life results and damage have the same pre-event history");
        Fired::PLAIN
    } else {
        None
    }
}

static DARIEN: CardDef = CardDef {
    name: "Damage Life Darien, King of Kjeldor",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(3), toughness: Some(3),
    abilities: &[triggered(when_asked(EventMask::DAMAGE, dealt_to_you), nothing,
        "Whenever you're dealt damage, you may create that many 1/1 white Soldier creature tokens.")],
    ..CardDef::BLANK
};

fn game() -> Game {
    let table = Box::leak(Box::new(CardTable::new([
        &WATCHER, &DOUBLER, &EMPERION, &REMEDY, &DRAIN, &PYROHEMIA, &LIFELINKER, &PRIDEMATE, &ANGEL, &DARIEN,
        &HAWK, &INFECT, &TOXIC, &ARMOR,
    ])));
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

fn pyrohemia_item(g: &Game, source: ObjectId) -> StackItem {
    let card = g.state.obj(source).card;
    crate::actions::stack_item(StackKind::Activated, source, AbilityRef { card, index: 0 }, g.state.obj(source).controller, &[])
}

#[test]
fn pyrohemia_triggers_ob_nixilis_once_for_three_opponents_losing_one() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let source = put(&mut g, &PYROHEMIA, YOU);
    let item = pyrohemia_item(&g, source);
    pyrohemia(&mut cx(&mut g), &item).unwrap();
    assert_eq!(life(&g), vec![19; 4]);
    assert_eq!(changes(&g), vec![(ACTIVE, -1), (LAST, -1), (YOU, -1), (FIRST, -1)]);
    assert_eq!(trigger_count(&g, watcher, 0), 1, "one simultaneous loss event, not three triggers");
    assert_eq!(trigger_count(&g, watcher, 1), 3, "Mindcrank still sees each opponent's occurrence");
}

#[test]
fn damage_and_its_life_results_share_the_same_trigger_history_boundary() {
    let mut g = game();
    let darien = put(&mut g, &DARIEN, FIRST);
    let source = put(&mut g, &PYROHEMIA, YOU);
    let item = pyrohemia_item(&g, source);
    pyrohemia(&mut cx(&mut g), &item).unwrap();
    assert_eq!(trigger_count(&g, darien, 0), 1);
    assert_eq!(g.state.obj(darien).damage, 1);
}

/// CR 510.2, 120.3a: two simultaneous one-damage assignments to one player cause one loss of two, not two losses of one.
#[test]
fn combat_damage_combines_a_players_losses_but_not_sources_lifelink() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let pridemate = put(&mut g, &PRIDEMATE, ACTIVE);
    let one = put(&mut g, &LIFELINKER, ACTIVE);
    let two = put(&mut g, &LIFELINKER, ACTIVE);
    let hits = [(one, Target::Player(FIRST), 1), (two, Target::Player(FIRST), 1)];
    let mut c = cx(&mut g);
    let mut batch = crate::ops::DamageBatch::new(&c, &hits);
    let mut events: Vec<_> = hits.into_iter()
        .filter_map(|(s, t, n)| crate::ops::deal_damage_with(&mut c, s, t, n, true, &mut batch)).collect();
    assert_eq!(life(c.g), vec![20; 4], "assignment preparation does not change life");
    batch.finish(&mut c, &mut events);
    c.emit_batch(&events, None);
    batch.rest_of_preventions(&mut c);
    assert_eq!(changes(&g), vec![(ACTIVE, 1), (ACTIVE, 1), (FIRST, -2)]);
    assert_eq!(trigger_count(&g, watcher, 0), 0);
    assert_eq!(trigger_count(&g, watcher, 1), 1);
    assert_eq!(trigger_count(&g, pridemate, 0), 2, "CR 702.15e: one gain per source, not per controller");
}

#[test]
fn lifelink_prepares_one_gain_per_source_across_all_its_recipients() {
    let mut g = game();
    let pridemate = put(&mut g, &PRIDEMATE, ACTIVE);
    put(&mut g, &ANGEL, ACTIVE);
    put(&mut g, &DOUBLER, ACTIVE);
    let one = put(&mut g, &LIFELINKER, ACTIVE);
    let two = put(&mut g, &LIFELINKER, ACTIVE);
    assert_eq!(cantrip_api::Game::damage_together(&mut cx(&mut g), &[
        (one, Target::Player(FIRST), 1), (one, Target::Player(LAST), 1), (two, Target::Player(FIRST), 1),
    ]), vec![1, 1, 1]);
    assert_eq!(changes(&g), vec![(ACTIVE, 3), (ACTIVE, 2), (LAST, -2), (FIRST, -4)]);
    assert_eq!(life(&g), vec![20, 16, 25, 18]);
    assert_eq!(trigger_count(&g, pridemate, 0), 2);
}

#[test]
fn single_damage_keeps_loss_and_lifelink_separate_even_at_unchanged_life() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let pridemate = put(&mut g, &PRIDEMATE, FIRST);
    let source = put(&mut g, &LIFELINKER, FIRST);
    assert_eq!(cantrip_api::Game::damage_result(&mut cx(&mut g), source, Target::Player(FIRST), 1),
        Some(DamageResult { target: Target::Player(FIRST), amount: 1, excess: 0 }));
    assert_eq!(life(&g), vec![20; 4]);
    assert_eq!(changes(&g), vec![(FIRST, -1), (FIRST, 1)], "do not net a loss against a gain");
    assert_eq!(trigger_count(&g, watcher, 0), 1);
    assert_eq!(trigger_count(&g, pridemate, 0), 1);
}

#[test]
fn damage_life_choices_precede_all_updates_and_survive_undo_redo() {
    let mut g = game();
    let one = put(&mut g, &DOUBLER, ACTIVE);
    let two = put(&mut g, &DOUBLER, ACTIVE);
    let source = put(&mut g, &PYROHEMIA, YOU);
    g.state.stack_push(pyrohemia_item(&g, source));
    let serial = g.state.stack.last().unwrap().serial;
    g.state.frame_push(Frame::new(FrameKind::Resolve { serial }));
    g.run();
    let before = g.at;
    for player in [LAST, YOU, FIRST] {
        let request = g.pending().cloned().expect("damage-result replacement question");
        assert_eq!(request.player, player, "life results are prepared in APNAP order");
        let RequestKind::ChooseReplacement { life: amount, sources, .. } = &request.kind else { panic!("{:?}", request.kind) };
        assert_eq!(*amount, Some(-1));
        assert_eq!(*sources, vec![one, two]);
        assert_eq!(life(&g), vec![20; 4]);
        assert!(changes(&g).is_empty());
        assert!(!g.log.iter().any(|r| matches!(r.event, Event::Damage { .. })));
        g.answer(request.id, player, Answer::Index(1)).expect("valid order");
        g.run();
    }
    let after = g.at;
    let expected = vec![(ACTIVE, -1), (LAST, -4), (YOU, -4), (FIRST, -4)];
    assert_eq!(changes(&g), expected);
    assert_eq!(life(&g), vec![16, 16, 19, 16]);
    assert_eq!(g.log.iter().filter(|r| matches!(r.event, Event::Damage { .. })).count(), 4);
    g.goto(before);
    assert_eq!(g.pending().unwrap().player, LAST);
    assert_eq!(life(&g), vec![20; 4]);
    assert!(changes(&g).is_empty());
    g.goto(after);
    assert_eq!(changes(&g), expected);
    assert_eq!(life(&g), vec![16, 16, 19, 16]);
    assert_eq!(g.log.iter().filter(|r| matches!(r.event, Event::Damage { .. })).count(), 4);
}

#[test]
fn damage_result_replacement_choices_rebuild_after_concession() {
    let mut g = game();
    put(&mut g, &DOUBLER, ACTIVE);
    put(&mut g, &DOUBLER, ACTIVE);
    let source = put(&mut g, &PYROHEMIA, YOU);
    g.state.stack_push(pyrohemia_item(&g, source));
    let serial = g.state.stack.last().unwrap().serial;
    g.state.frame_push(Frame::new(FrameKind::Resolve { serial }));
    g.run();
    assert_eq!(g.pending().unwrap().player, LAST);
    assert!(matches!(g.pending().unwrap().kind, RequestKind::ChooseReplacement { .. }));
    g.concede(ACTIVE);
    assert!(!matches!(g.pending().map(|r| &r.kind), Some(RequestKind::ChooseReplacement { .. })));
    assert_eq!(changes(&g), vec![(LAST, -1), (YOU, -1), (FIRST, -1)]);
    assert_eq!(life(&g), vec![19, 19, 20, 19]);
    // CR 120.4b-c, 104.3a: all four assignments were dealt before this life-result choice. Concession preserves that
    // damage, but the departing player has no subsequent life change. Replay must not move the concession earlier.
    let recipients: Vec<_> = g.log.iter().filter_map(|r| match r.event {
        Event::Damage { target, .. } => Some(target),
        _ => None,
    }).collect();
    assert_eq!(recipients, vec![Target::Player(ACTIVE), Target::Player(LAST), Target::Player(YOU), Target::Player(FIRST)]);
}

#[test]
fn combat_step_uses_one_life_batch_for_multiple_opponents_and_lifelink_sources() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, ACTIVE);
    let pridemate = put(&mut g, &PRIDEMATE, ACTIVE);
    let mut attackers = Vec::new();
    for player in [YOU, FIRST, LAST] {
        let hawk = put(&mut g, &HAWK, ACTIVE);
        attackers.push(Attacker::new(hawk, Target::Player(player), player, TypeSet::default()));
    }
    g.state.set_attackers(attackers);
    g.state.update_turn(|t| t.set_step(Step::CombatDamage));
    crate::turn::begin_step(&mut cx(&mut g)).unwrap();
    assert_eq!(life(&g), vec![19, 19, 23, 19]);
    assert_eq!(trigger_count(&g, watcher, 0), 1);
    assert_eq!(trigger_count(&g, watcher, 1), 3);
    assert_eq!(trigger_count(&g, pridemate, 0), 3);
    assert_eq!(g.log.iter().filter(|r| matches!(r.event, Event::Damage { kind, .. } if kind.is_combat())).count(), 3);
}

#[test]
fn sign_changed_lifelink_and_damage_losses_are_announced_together() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let pridemate = put(&mut g, &PRIDEMATE, FIRST);
    put(&mut g, &REMEDY, YOU);
    let source = put(&mut g, &LIFELINKER, FIRST);
    cantrip_api::Game::damage(&mut cx(&mut g), source, Target::Player(LAST), 1);
    assert_eq!(changes(&g), vec![(LAST, -1), (FIRST, -1)]);
    assert_eq!(trigger_count(&g, watcher, 0), 1, "Tainted Remedy's result belongs to the same damage event");
    assert_eq!(trigger_count(&g, watcher, 1), 2);
    assert_eq!(trigger_count(&g, pridemate, 0), 0);
}

#[test]
fn prohibited_life_results_do_not_replace_or_prevent_the_damage_itself() {
    for block_gain in [false, true] {
        let mut g = game();
        put(&mut g, &EMPERION, FIRST);
        put(&mut g, &DOUBLER, ACTIVE);
        put(&mut g, &DOUBLER, ACTIVE);
        put(&mut g, &ANGEL, ACTIVE);
        let source = put(&mut g, &LIFELINKER, ACTIVE);
        if block_gain {
            put(&mut g, &REMEDY, YOU);
            g.state.effect_push(ContinuousEffect {
                source, affected: source, controller: ACTIVE, timestamp: 1, words: Words::NONE,
                kind: EffectKind::CantGainLife(ACTIVE), duration: Duration::EndOfTurn,
            });
        }
        assert_eq!(cantrip_api::Game::damage(&mut cx(&mut g), source, Target::Player(FIRST), 2), 2);
        assert_eq!(g.state.player(FIRST).life, 20);
        assert_eq!(changes(&g), if block_gain { vec![] } else { vec![(ACTIVE, 3)] });
        assert!(g.state.agenda[0].pending.is_none(), "prohibited results offer no replacement choices");
        assert_eq!(g.log.iter().filter(|r| matches!(r.event, Event::Damage { amount: 2, .. })).count(), 1);
    }
}

#[test]
fn damage_life_results_use_only_the_damage_left_after_prevention() {
    let mut g = game();
    g.state.update_turn(|t| t.number = 1);
    let watcher = put(&mut g, &WATCHER, YOU);
    put(&mut g, &ARMOR, FIRST);
    put(&mut g, &ANGEL, ACTIVE);
    let one = put(&mut g, &LIFELINKER, ACTIVE);
    let two = put(&mut g, &LIFELINKER, ACTIVE);
    assert_eq!(cantrip_api::Game::damage_together(&mut cx(&mut g), &[
        (one, Target::Player(FIRST), 1), (two, Target::Player(FIRST), 2),
    ]), vec![0, 1]);
    assert_eq!(changes(&g), vec![(ACTIVE, 2), (FIRST, -1)]);
    assert_eq!(trigger_count(&g, watcher, 0), 1);
    assert_eq!(g.log.iter().filter(|r| matches!(r.event, Event::DamagePrevented { amount: 1, .. })).count(), 2);
}

#[test]
fn combat_infect_and_toxic_keep_their_poison_results_alongside_life_and_lifelink() {
    let mut g = game();
    let watcher = put(&mut g, &WATCHER, YOU);
    let mut attackers = Vec::new();
    for def in [&INFECT, &TOXIC, &HAWK] {
        let source = put(&mut g, def, ACTIVE);
        attackers.push(Attacker::new(source, Target::Player(FIRST), FIRST, TypeSet::default()));
    }
    g.state.set_attackers(attackers);
    g.state.update_turn(|t| t.set_step(Step::CombatDamage));
    crate::turn::begin_step(&mut cx(&mut g)).unwrap();
    assert_eq!(changes(&g), vec![(ACTIVE, 1), (FIRST, -2)]);
    assert_eq!(g.state.player(FIRST).poison(), 2, "one infect counter and one toxic counter");
    assert_eq!(trigger_count(&g, watcher, 0), 0, "only noninfect damage contributes to the combined loss");
    assert_eq!(trigger_count(&g, watcher, 1), 1);
}

#[test]
fn simultaneous_damage_life_results_keep_large_totals_at_full_width() {
    let mut g = game();
    let source = put(&mut g, &LIFELINKER, ACTIVE);
    assert_eq!(cantrip_api::Game::damage_together(&mut cx(&mut g), &[
        (source, Target::Player(FIRST), 40_000), (source, Target::Player(FIRST), 40_000),
    ]), vec![40_000, 40_000]);
    assert_eq!(changes(&g), vec![(ACTIVE, 80_000), (FIRST, -80_000)]);
    assert_eq!(g.state.player(FIRST).life, -79_980);
    assert_eq!(g.state.player(ACTIVE).life, 80_020);
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
