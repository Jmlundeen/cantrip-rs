//! Read ahead (CR 702.155, 714.3b) as the core plays it, over test cards: as a Saga with read ahead enters - cast, put onto
//! the battlefield, or created as a token copy - its controller chooses a number from one to its final chapter number, and
//! it enters with that many lore counters (CR 702.155b) in place of a Saga's one (CR 714.3a); its chapter abilities can't
//! trigger the turn it entered unless it has exactly the chapter's number of lore counters (CR 702.155a), so the chapters it
//! skips don't trigger, nor do those counters put on it later that turn pass over; on a later turn they trigger as ever
//! (CR 714.2b). One that starts on its final chapter is sacrificed once that chapter has left the stack (CR 714.4). Read
//! ahead a static ability gives it (Barbara Wright's "Sagas you control have read ahead") counts as it enters (CR 614.12).

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Read Ahead Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn gain_one(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    Ok(())
}

fn gain_ten(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 10);
    Ok(())
}

fn gain_hundred(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 100);
    Ok(())
}

/// "Read ahead / I - You gain 1 life. / II - You gain 10 life. / III - You gain 100 life." for {0}: the life gained says
/// which chapters triggered.
static SAGA: CardDef = CardDef {
    name: "Read Ahead Test Saga",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Enchantment]),
    subtypes: SubtypeSet::of(&[Subtype::Saga]),
    keywords: Keywords::of(&[Keyword::ReadAhead]),
    abilities: &[
        chapter(1, gain_one, "I \u{2014} You gain 1 life."),
        chapter(2, gain_ten, "II \u{2014} You gain 10 life."),
        chapter(3, gain_hundred, "III \u{2014} You gain 100 life."),
    ],
    ..CardDef::BLANK
};

/// The same Saga without read ahead.
static PLAIN_SAGA: CardDef = CardDef { name: "Read Ahead Test Plain Saga", keywords: Keywords::NONE, ..SAGA };

fn sagas_you_control(q: &dyn LayerQuery, source: ObjectId, object: ObjectId, chars: &Chars) -> bool {
    chars.has_subtype(Subtype::Saga) && q.state().obj(object).controller == q.state().obj(source).controller
}

fn give_read_ahead(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::ReadAhead);
}

/// "Sagas you control have read ahead." (Barbara Wright's.) An enchantment for {0}.
static GRANTER: CardDef = CardDef {
    name: "Read Ahead Test Granter",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Ability, sagas_you_control, give_read_ahead, "Sagas you control have read ahead.")],
    ..CardDef::BLANK
};

fn sagas_of(g: &dyn cantrip_api::Game, player: PlayerId) -> Vec<ObjectId> {
    g.state().battlefield().iter().copied().filter(|&id| g.controller(id) == player && g.chars(id).has_subtype(Subtype::Saga)).collect()
}

fn two_lore_counters(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    for saga in sagas_of(g, item.controller) {
        g.add_counters(saga, CounterKind::Lore, 2);
    }
    Ok(())
}

fn put_saga_from_hand(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let saga = g.state().zone(Zone::hand(item.controller)).iter().copied().find(|&id| g.chars(id).has_subtype(Subtype::Saga));
    if let Some(saga) = saga {
        g.put_onto_battlefield(saga, item.controller, false)?;
    }
    Ok(())
}

fn copy_a_saga(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(&saga) = sagas_of(g, item.controller).first() {
        g.create_token_copies(saga, item.controller, 1)?;
    }
    Ok(())
}

/// "Put two lore counters on each Saga you control." An instant for {0}.
static TWO_LORE: CardDef = CardDef {
    name: "Read Ahead Test Two Lore",
    cost: ManaCost::ZERO,
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef { targets: &[], effect: two_lore_counters }),
    ..CardDef::BLANK
};

/// "Put a Saga card from your hand onto the battlefield."
static PUT_SAGA: CardDef = CardDef { name: "Read Ahead Test Put", spell: Some(SpellDef { targets: &[], effect: put_saga_from_hand }), ..TWO_LORE };

/// "Create a token that's a copy of a Saga you control."
static COPY_SAGA: CardDef = CardDef { name: "Read Ahead Test Copy", spell: Some(SpellDef { targets: &[], effect: copy_a_saga }), ..TWO_LORE };

fn give_saga_to_them(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let saga = g.state().zone(Zone::hand(item.controller)).iter().copied().find(|&id| g.chars(id).has_subtype(Subtype::Saga));
    if let Some(saga) = saga {
        g.put_onto_battlefield(saga, THEM, false)?;
    }
    Ok(())
}

fn flicker_a_saga(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(&saga) = sagas_of(g, item.controller).first() {
        let exiled = g.exile(saga);
        g.put_onto_battlefield(exiled, item.controller, false)?;
    }
    Ok(())
}

/// "Put a Saga card from your hand onto the battlefield under an opponent's control."
static GIVE_SAGA: CardDef = CardDef { name: "Read Ahead Test Give", spell: Some(SpellDef { targets: &[], effect: give_saga_to_them }), ..TWO_LORE };

/// "Exile a Saga you control, then return it to the battlefield under your control."
static FLICKER_SAGA: CardDef = CardDef { name: "Read Ahead Test Flicker", spell: Some(SpellDef { targets: &[], effect: flicker_a_saga }), ..TWO_LORE };

fn saga(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.has_subtype(Subtype::Saga)
}

/// "Sagas lose all abilities."
static HUMBLER: CardDef = CardDef {
    name: "Read Ahead Test Humbler",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[loses_all_abilities(saga, "Sagas lose all abilities.")],
    ..CardDef::BLANK
};

/// Your precombat main phase of turn 3 with priority and an empty stack, every answer round-tripped through the game-log
/// tree (`Game::verify`).
fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([
        &FILLER,
        &SAGA,
        &PLAIN_SAGA,
        &GRANTER,
        &TWO_LORE,
        &PUT_SAGA,
        &COPY_SAGA,
        &GIVE_SAGA,
        &FLICKER_SAGA,
        &HUMBLER,
    ])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.number = 3;
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g.verify = true;
    g
}

fn put(g: &mut Game, def: &'static CardDef, zone: Zone) -> ObjectId {
    put_for(g, def, zone, YOU)
}

/// `put`, owned and controlled by `player`.
fn put_for(g: &mut Game, def: &'static CardDef, zone: Zone, player: PlayerId) -> ObjectId {
    let mut obj = Obj::new(g.cards.id_of(def), player, zone);
    obj.timestamp = g.state.timestamp();
    let id = g.state.push_obj(obj);
    g.state.zone_push(zone, id);
    if !def.statics.is_empty() {
        g.state.see_statics();
    }
    id
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

/// Casts `card` from your hand.
fn cast(g: &mut Game, card: ObjectId) {
    let r = next(g);
    let RequestKind::Priority { options } = &r.kind else { panic!("expected priority, got {:?}", r.kind) };
    let option = options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)).expect("it may be cast").clone();
    g.answer(r.id, r.player, Answer::Act(Action::of(&option))).expect("a legal answer");
    loop {
        let r = next(g);
        let RequestKind::PayCosts { options, .. } = &r.kind else { break };
        let pay = options.iter().find(|o| matches!(o.kind, ActionKind::ConfirmCasting | ActionKind::AssignMana)).expect("it costs nothing").clone();
        g.answer(r.id, r.player, Answer::Act(Action::of(&pay))).expect("a legal answer");
    }
}

/// Passes until the stack is empty with you to act, answering each read-ahead choice with the next of `choices`: the
/// ranges each offered, and to whom.
fn resolve(g: &mut Game, choices: &[u32]) -> Vec<(PlayerId, u32, u32)> {
    let mut choices = choices.iter();
    let mut offered = Vec::new();
    loop {
        let r = next(g);
        match &r.kind {
            RequestKind::Priority { .. } if g.state.stack.is_empty() && g.state.triggers.is_empty() && r.player == YOU => break,
            RequestKind::Priority { .. } => g.answer(r.id, r.player, Answer::Pass).expect("a legal answer"),
            RequestKind::ChooseNumber { least, most, .. } => {
                offered.push((r.player, *least, *most));
                let n = *choices.next().expect("a choice to make");
                g.answer(r.id, r.player, Answer::Announce(n)).expect("a number offered");
            }
            _ => answer_other(g, &r),
        }
    }
    assert!(choices.next().is_none(), "every choice was asked");
    offered
}

/// The questions beside priority and read ahead's that the games here ask: triggers in the order found, a discard to hand
/// size of the first cards.
fn answer_other(g: &mut Game, r: &Request) {
    let answer = match &r.kind {
        RequestKind::OrderTriggers { sources, .. } => Answer::Order((0..sources.len() as u32).collect()),
        RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
        other => panic!("unexpected {other:?}"),
    };
    g.answer(r.id, r.player, answer).expect("a legal answer");
}

fn life(g: &Game) -> i32 {
    i32::from(g.state.player(YOU).life)
}

fn lore(g: &Game, id: ObjectId) -> i32 {
    g.state.obj(id).counters(CounterKind::Lore)
}

/// The permanent `card` became, if it is on the battlefield.
fn on_battlefield(g: &Game, card: ObjectId) -> Option<ObjectId> {
    let mut new = card;
    while let Some(next) = g.state.obj(new).became {
        new = next;
    }
    g.state.obj(new).zone.is(ZoneKind::Battlefield).then_some(new)
}

/// Casts the read-ahead Saga, choosing `n`: the permanent, if it stays, and the ranges offered.
fn cast_saga(n: u32) -> (Game, Option<ObjectId>, Vec<(PlayerId, u32, u32)>) {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    let offered = resolve(&mut g, &[n]);
    let permanent = on_battlefield(&g, saga);
    (g, permanent, offered)
}

/// CR 702.155b: one to the final chapter number is offered its controller; chosen 1, it enters with one lore counter and
/// chapter I triggers (CR 714.2b), as for any Saga.
#[test]
fn starting_on_chapter_one_triggers_chapter_one() {
    let (g, saga, offered) = cast_saga(1);
    assert_eq!(offered, vec![(YOU, 1, 3)], "one to its final chapter, III, asked of its controller");
    let saga = saga.expect("it stays");
    assert_eq!(lore(&g, saga), 1);
    assert_eq!(life(&g), 21, "chapter I alone");
}

/// CR 702.155a-b: chosen 2, it enters with two lore counters; chapter I is skipped (it has not exactly one) and doesn't
/// trigger, chapter II does - not one plus the one a Saga without read ahead has (CR 714.3a).
#[test]
fn starting_on_chapter_two_skips_chapter_one() {
    let (g, saga, _) = cast_saga(2);
    let saga = saga.expect("it stays");
    assert_eq!(lore(&g, saga), 2);
    assert_eq!(life(&g), 30, "chapter II alone");
}

/// CR 702.155a, 714.4: chosen its final chapter, only that chapter triggers, and the Saga is sacrificed once it has left
/// the stack.
#[test]
fn starting_on_the_final_chapter_triggers_it_and_then_the_saga_is_sacrificed() {
    let (g, saga, _) = cast_saga(3);
    assert_eq!(saga, None, "sacrificed");
    assert_eq!(life(&g), 120, "chapter III alone, resolved before the sacrifice");
    assert_eq!(g.state.zone(Zone::graveyard(YOU)).len(), 1);
}

/// CR 714.3a: a Saga without read ahead is asked nothing and enters with one lore counter; counters put on it the same turn
/// trigger every chapter they reach (CR 714.2b) - the control for read ahead's restriction below.
#[test]
fn a_saga_without_read_ahead_chooses_nothing_and_its_chapters_trigger_as_reached() {
    let mut g = game();
    let saga = put(&mut g, &PLAIN_SAGA, Zone::hand(YOU));
    let more = put(&mut g, &TWO_LORE, Zone::hand(YOU));
    cast(&mut g, saga);
    assert_eq!(resolve(&mut g, &[]), vec![], "no choice");
    let saga = on_battlefield(&g, saga).expect("it stays");
    assert_eq!(lore(&g, saga), 1);
    assert_eq!(life(&g), 21);
    cast(&mut g, more);
    resolve(&mut g, &[]);
    assert_eq!(life(&g), 131, "II and III both triggered");
}

/// CR 702.155a: the turn it entered, a chapter triggers only with exactly its number of lore counters - two more counters
/// on a Saga that started on chapter I (three) trigger III, but not II, which they passed over.
#[test]
fn counters_put_on_it_the_turn_it_entered_trigger_only_the_chapter_they_land_on() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let more = put(&mut g, &TWO_LORE, Zone::hand(YOU));
    cast(&mut g, saga);
    resolve(&mut g, &[1]);
    assert_eq!(life(&g), 21);
    cast(&mut g, more);
    resolve(&mut g, &[]);
    assert_eq!(life(&g), 121, "III alone");
    assert_eq!(on_battlefield(&g, saga), None, "sacrificed after its final chapter");
}

/// CR 702.155a, 714.2b: on a later turn - the opponent's next - its chapters trigger as any Saga's: two more counters on one
/// that started on chapter I trigger II and III.
#[test]
fn on_a_later_turn_its_chapters_trigger_as_reached() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let more = put(&mut g, &TWO_LORE, Zone::hand(YOU));
    cast(&mut g, saga);
    resolve(&mut g, &[1]);
    let permanent = on_battlefield(&g, saga).expect("it stays");
    // On to the opponent's turn, where you next have priority.
    loop {
        let r = next(&mut g);
        if g.state.turn.active == THEM && r.player == YOU && matches!(r.kind, RequestKind::Priority { .. }) {
            break;
        }
        match &r.kind {
            RequestKind::Priority { .. } => g.answer(r.id, r.player, Answer::Pass).expect("a legal answer"),
            _ => answer_other(&mut g, &r),
        }
    }
    assert_eq!(lore(&g, permanent), 1);
    cast(&mut g, more);
    // Pass until the stack is empty again, on their turn.
    loop {
        let r = next(&mut g);
        match &r.kind {
            RequestKind::Priority { .. } if g.state.stack.is_empty() && g.state.triggers.is_empty() => break,
            RequestKind::Priority { .. } => g.answer(r.id, r.player, Answer::Pass).expect("a legal answer"),
            _ => answer_other(&mut g, &r),
        }
    }
    assert_eq!(g.state.turn.active, THEM);
    assert_eq!(life(&g), 131, "II and III");
}

/// CR 702.155b, 614.12: put onto the battlefield without being cast, it enters all the same, and chooses.
#[test]
fn put_onto_the_battlefield_it_chooses_as_it_enters() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let probe = put(&mut g, &PUT_SAGA, Zone::hand(YOU));
    cast(&mut g, probe);
    assert_eq!(resolve(&mut g, &[2]), vec![(YOU, 1, 3)]);
    let saga = on_battlefield(&g, saga).expect("it entered");
    assert_eq!(lore(&g, saga), 2);
    assert_eq!(life(&g), 30, "chapter II alone");
}

/// CR 707.5, 614.12: a token copy of a read-ahead Saga has read ahead, and chooses its own number as it is created - here
/// its final chapter, so it gains 100 and is sacrificed, while the original, started on chapter I, stays.
#[test]
fn a_token_copy_chooses_its_own_number() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let probe = put(&mut g, &COPY_SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    resolve(&mut g, &[1]);
    cast(&mut g, probe);
    assert_eq!(resolve(&mut g, &[3]), vec![(YOU, 1, 3)]);
    assert_eq!(life(&g), 121);
    let saga = on_battlefield(&g, saga).expect("the original stays");
    assert_eq!(lore(&g, saga), 1);
    assert_eq!(g.state.battlefield().to_vec(), vec![saga], "the token was sacrificed after its final chapter");
}

/// CR 614.12: read ahead a static ability would give it on the battlefield is its as it enters - a Saga without read ahead
/// of its own chooses, and skips what it skips (CR 702.155a).
#[test]
fn read_ahead_a_static_ability_gives_counts_as_it_enters() {
    let mut g = game();
    put(&mut g, &GRANTER, Zone::BATTLEFIELD);
    let saga = put(&mut g, &PLAIN_SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    assert_eq!(resolve(&mut g, &[2]), vec![(YOU, 1, 3)]);
    let saga = on_battlefield(&g, saga).expect("it stays");
    assert_eq!(lore(&g, saga), 2);
    assert_eq!(life(&g), 30, "chapter II alone");
}

/// CR 702.155a: read ahead given to a Saga after it entered restricts its chapters that turn all the same - "the turn it
/// entered" is the Saga's, whenever it came to have read ahead. Two more counters on a plain Saga that started on chapter I
/// trigger III alone once a static ability gives it read ahead (without one, II and III: the control above).
#[test]
fn read_ahead_given_after_it_entered_restricts_its_chapters_that_turn() {
    let mut g = game();
    let saga = put(&mut g, &PLAIN_SAGA, Zone::hand(YOU));
    let granter = put(&mut g, &GRANTER, Zone::hand(YOU));
    let more = put(&mut g, &TWO_LORE, Zone::hand(YOU));
    cast(&mut g, saga);
    assert_eq!(resolve(&mut g, &[]), vec![], "no read ahead as it entered, no choice");
    assert_eq!(life(&g), 21);
    cast(&mut g, granter);
    resolve(&mut g, &[]);
    let permanent = on_battlefield(&g, saga).expect("it stays");
    assert!(g.chars(permanent).has(Keyword::ReadAhead), "given read ahead");
    cast(&mut g, more);
    resolve(&mut g, &[]);
    assert_eq!(life(&g), 121, "III alone");
}

/// CR 614.12: a static ability that gives read ahead to other Sagas than this one - the opponent's "Sagas you control" -
/// gives it none: it chooses nothing, and its chapters trigger as reached.
#[test]
fn read_ahead_given_only_to_others_is_not_its() {
    let mut g = game();
    put_for(&mut g, &GRANTER, Zone::BATTLEFIELD, THEM);
    let saga = put(&mut g, &PLAIN_SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    assert_eq!(resolve(&mut g, &[]), vec![], "no choice");
    let saga = on_battlefield(&g, saga).expect("it stays");
    assert_eq!(lore(&g, saga), 1);
    assert_eq!(life(&g), 21);
}

/// CR 614.12, 702.155b: put onto the battlefield under an opponent's control, the Saga's choice is the player it enters
/// under, not its owner; its chapter is theirs (CR 714.2b).
#[test]
fn the_choice_is_the_player_it_enters_under() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let probe = put(&mut g, &GIVE_SAGA, Zone::hand(YOU));
    cast(&mut g, probe);
    assert_eq!(resolve(&mut g, &[2]), vec![(THEM, 1, 3)]);
    let saga = on_battlefield(&g, saga).expect("it entered");
    assert_eq!(g.state.obj(saga).controller, THEM);
    assert_eq!(lore(&g, saga), 2);
    assert_eq!(life(&g), 20);
    assert_eq!(i32::from(g.state.player(THEM).life), 30, "chapter II alone, theirs");
}

/// CR 614.12: a Saga that would have lost all its abilities on the battlefield has no read ahead as it enters, and chooses
/// nothing.
#[test]
fn a_saga_that_would_lose_its_abilities_chooses_nothing() {
    let mut g = game();
    put(&mut g, &HUMBLER, Zone::BATTLEFIELD);
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    assert_eq!(resolve(&mut g, &[]), vec![], "no choice");
    assert_eq!(life(&g), 20, "no chapter abilities");
}

/// CR 400.7: a read-ahead Saga that leaves and returns is a new object, which chooses again as it enters, and is restricted
/// as one that entered this turn: returned on chapter I, two more counters trigger III alone.
#[test]
fn returned_to_the_battlefield_it_chooses_again() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    let flicker = put(&mut g, &FLICKER_SAGA, Zone::hand(YOU));
    let more = put(&mut g, &TWO_LORE, Zone::hand(YOU));
    cast(&mut g, saga);
    resolve(&mut g, &[2]);
    assert_eq!(life(&g), 30, "chapter II");
    let first = on_battlefield(&g, saga).expect("it stays");
    cast(&mut g, flicker);
    assert_eq!(resolve(&mut g, &[1]), vec![(YOU, 1, 3)], "asked again");
    let second = on_battlefield(&g, saga).expect("it returned");
    assert_ne!(first, second, "a new object");
    assert_eq!(lore(&g, second), 1);
    assert_eq!(life(&g), 31, "chapter I of the new object");
    cast(&mut g, more);
    resolve(&mut g, &[]);
    assert_eq!(life(&g), 131, "III alone");
}

/// The choice is an answer in the game-log tree (`Game::goto`): back to before it, the question is asked again, and another
/// answer makes another game; forward again, the first is as it was.
#[test]
fn the_choice_is_undone_and_redone_through_the_game_tree() {
    let mut g = game();
    let saga = put(&mut g, &SAGA, Zone::hand(YOU));
    cast(&mut g, saga);
    // Pass until the read-ahead choice is asked.
    let asked = loop {
        let r = next(&mut g);
        match &r.kind {
            RequestKind::Priority { .. } => g.answer(r.id, r.player, Answer::Pass).expect("a legal answer"),
            RequestKind::ChooseNumber { .. } => break r,
            other => panic!("unexpected {other:?}"),
        }
    };
    let before = g.at;
    g.answer(asked.id, asked.player, Answer::Announce(3)).expect("a number offered");
    resolve(&mut g, &[]);
    let after = g.at;
    assert_eq!(life(&g), 120);
    g.goto(before);
    assert_eq!(life(&g), 20);
    let again = g.pending().cloned().expect("asked again");
    assert!(matches!(again.kind, RequestKind::ChooseNumber { least: 1, most: 3, .. }), "{:?}", again.kind);
    g.answer(again.id, again.player, Answer::Announce(1)).expect("a number offered");
    resolve(&mut g, &[]);
    assert_eq!(life(&g), 21);
    assert_eq!(lore(&g, on_battlefield(&g, saga).expect("it stays")), 1);
    g.goto(after);
    assert_eq!(life(&g), 120);
    assert_eq!(on_battlefield(&g, saga), None);
}
