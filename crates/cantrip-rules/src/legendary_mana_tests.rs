//! Mana spendable only on a legendary spell (CR 106.6), over test cards, since no card in the engine prints it alone yet:
//! it casts a legendary spell, is no help to any other, and does nothing to the spell it pays for. Delighted Halfling's
//! is the same limit, with an effect, and is mana of its own kind.

use crate::{Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

static FILLER: CardDef = CardDef { name: "Legendary Mana Test Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

/// A legendary 2/2 for {G}.
static LEGEND: CardDef = CardDef {
    name: "Legendary Mana Test Legend",
    cost: mana("{G}"),
    types: TypeSet::of(&[CardType::Creature]),
    supertypes: Supertypes { basic: false, legendary: true, snow: false },
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// The same creature, not legendary.
static COMMONER: CardDef = CardDef { name: "Legendary Mana Test Commoner", supertypes: Supertypes { basic: false, legendary: false, snow: false }, ..LEGEND };

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &LEGEND, &COMMONER])));
    let deck = || vec![table.id_of(&FILLER); 20];
    let mut g = Game::new(table, &Setup { decks: vec![deck(), deck()], ..Default::default() });
    while !g.state.agenda.is_empty() {
        g.state.frame_remove(0);
    }
    g.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    g.state.frame_push(Frame::new(FrameKind::Priority));
    g
}

fn in_hand(g: &mut Game, def: &'static CardDef) -> ObjectId {
    let zone = Zone::hand(YOU);
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), YOU, zone));
    g.state.zone_push(zone, id);
    id
}

fn next(g: &mut Game) -> Request {
    g.run();
    g.pending().cloned().expect("a question")
}

fn cast_option(g: &mut Game, card: ObjectId) -> Option<ActionOption> {
    match next(g).kind {
        RequestKind::Priority { options } => options.into_iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(card)),
        other => panic!("expected priority, got {other:?}"),
    }
}

/// Casts the card with the restricted {G} in the pool, and hands back the spell.
fn cast_with(restriction: ManaRestriction) -> (Game, ObjectId) {
    let mut g = game();
    let card = in_hand(&mut g, &LEGEND);
    let source = g.state.zone(Zone::library(YOU))[0];
    g.state.add_mana(YOU, Mana::G, 1, source, Some(restriction), ManaLasts::Step);
    let option = cast_option(&mut g, card).expect("the mana casts a legendary spell");
    let r = next(&mut g);
    g.answer(r.id, r.player, Answer::Act(Action::of(&option))).expect("a legal answer");
    loop {
        let r = next(&mut g);
        let RequestKind::PayCosts { options, .. } = &r.kind else { break };
        let pay = options.iter().find(|o| matches!(o.kind, ActionKind::ConfirmCasting | ActionKind::AssignMana)).expect("the pool pays").clone();
        g.answer(r.id, r.player, Answer::Act(Action::of(&pay))).expect("a legal answer");
    }
    let spell = g.state.stack.last().expect("the spell").object;
    assert_eq!(g.state.player(YOU).pool.total() + g.state.restricted_of(YOU).map(|(_, pool)| pool.total()).sum::<u32>(), 0, "the mana is spent");
    (g, spell)
}

/// CR 106.6: "spend this mana only to cast a legendary spell", and that is all it says: the spell can be countered.
#[test]
fn it_casts_a_legendary_spell_and_does_nothing_to_it() {
    let (g, spell) = cast_with(ManaRestriction::LegendarySpells);
    assert!(!g.state.obj(spell).marks.cant_be_countered && !g.state.obj(spell).marks.gains_haste);
}

/// The positive control: the same payment with Delighted Halfling's mana is what makes a spell uncounterable.
#[test]
fn the_mana_that_also_says_it_cant_be_countered_does_that() {
    let (g, spell) = cast_with(ManaRestriction::LegendaryUncounterable);
    assert!(g.state.obj(spell).marks.cant_be_countered);
}

/// CR 106.6: it is no help to a spell that is not legendary, so with nothing else that spell is not offered.
#[test]
fn it_does_not_cast_a_spell_that_is_not_legendary() {
    let mut g = game();
    let commoner = in_hand(&mut g, &COMMONER);
    let source = g.state.zone(Zone::library(YOU))[0];
    g.state.add_mana(YOU, Mana::G, 1, source, Some(ManaRestriction::LegendarySpells), ManaLasts::Step);
    assert!(cast_option(&mut g, commoner).is_none());
}
