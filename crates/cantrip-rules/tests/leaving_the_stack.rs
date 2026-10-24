//! A spell's own "exile it instead" as it leaves the stack (CR 614.1a): flashback's "exile this card instead of putting it
//! anywhere else any time it would leave the stack" (CR 702.34a), and the "if that spell would be put into your graveyard,
//! exile it instead" an effect that has a spell cast makes (`ExileInstead`) - each exactly as worded: no type condition
//! unless the text has one, no destination but the one it names, and ordered by the spell's controller with the other
//! replacement effects that would change the same move (CR 616.1). The cards here are probes, not card implementations.

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game as RulesGame, Setup};
use cantrip_state::{Frame, FrameKind};

const ME: PlayerId = PlayerId(0);

static INSTANT: CardDef = CardDef {
    name: "Leave Instant",
    oracle: true,
    cost: ManaCost { generic: 1, ..ManaCost::ZERO },
    has_cost: true,
    types: TypeSet::of(&[CardType::Instant]),
    spell: Some(SpellDef::MODAL),
    ..CardDef::BLANK
};

/// A permanent card an effect may cast from a graveyard, as Dreadhorde Arcanist may the creature face of a card, or Bilbo an
/// artifact.
static ARTIFACT: CardDef = CardDef { name: "Leave Artifact", types: TypeSet::of(&[CardType::Artifact]), spell: None, ..INSTANT };

static FLASHBACK_WAY: WayDef = cantrip_api::way(Cost::mana(ManaCost::ZERO), ZoneKind::Graveyard, "Flashback {0}").exile_after();

static FLASHBACK_INSTANT: CardDef = CardDef { name: "Leave Flashback Instant", ways: &[FLASHBACK_WAY], ..INSTANT };

/// The first card of its controller's graveyard, cast for nothing on the terms `exile_instead` says, then sent off the stack
/// as `then` says (`None`: left to resolve).
fn cast_then(g: &mut dyn cantrip_api::Game, item: &StackItem, exile_instead: ExileInstead, then: Option<CounterTo>, counter: bool) -> Result<(), Suspend> {
    let Some(&card) = g.state().zone(Zone::graveyard(item.controller)).first() else { return Ok(()) };
    g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), exile_instead, ..CastTerms::default() })?;
    let Some(spell) = g.state().stack.last().map(|i| i.object).filter(|&s| s != item.object) else { return Ok(()) };
    if counter {
        g.counter(spell);
    } else if let Some(to) = then {
        g.remove_spell(spell, to);
    }
    Ok(())
}

fn counter_any(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    cast_then(g, item, ExileInstead::Spell, None, true)
}

fn counter_instant_or_sorcery(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    cast_then(g, item, ExileInstead::InstantOrSorcery, None, true)
}

fn bounce_any(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    cast_then(g, item, ExileInstead::Spell, Some(CounterTo::Hand), false)
}

fn bottom_any(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    cast_then(g, item, ExileInstead::Spell, Some(CounterTo::LibraryBottom), false)
}

/// The first card of its controller's graveyard cast for nothing as it is, then countered: a flashback card by its flashback.
fn counter_by_way(g: &mut dyn cantrip_api::Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&card) = g.state().zone(Zone::graveyard(item.controller)).first() else { return Ok(()) };
    g.cast_during_resolution_as(card, CastTerms::default())?;
    if let Some(spell) = g.state().stack.last().map(|i| i.object).filter(|&s| s != item.object) {
        g.counter(spell);
    }
    Ok(())
}

macro_rules! stager {
    ($name:literal, $effect:ident) => {
        CardDef { name: $name, cost: ManaCost::ZERO, spell: Some(SpellDef { targets: &[], effect: $effect }), ..INSTANT }
    };
}

static COUNTER_ANY: CardDef = stager!("Leave Counter Any", counter_any);
static COUNTER_INSTANT_OR_SORCERY: CardDef = stager!("Leave Counter Instant Or Sorcery", counter_instant_or_sorcery);
static BOUNCE_ANY: CardDef = stager!("Leave Bounce Any", bounce_any);
static BOTTOM_ANY: CardDef = stager!("Leave Bottom Any", bottom_any);
static COUNTER_BY_WAY: CardDef = stager!("Leave Counter By Way", counter_by_way);

/// "If a card would be put into a graveyard from anywhere, put it on the bottom of its owner's library instead" (Wheel of
/// Sun and Moon's kind).
fn to_a_graveyard(_: &dyn Query, _: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::ZoneChange(z) if z.is_card && z.to.is(ZoneKind::Graveyard))
}

fn to_its_library(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::ZoneChange(z) = proposal {
        z.to = Zone::library(z.to.player);
    }
}

static RECYCLER: CardDef = CardDef {
    name: "Leave Recycler",
    types: TypeSet::of(&[CardType::Enchantment]),
    replacements: &[replacement(ReplacementKind::ZoneChange, to_a_graveyard, to_its_library, "put it into its library instead")],
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Leave Filler", has_cost: false, ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| {
        CardTable::new([
            &INSTANT,
            &ARTIFACT,
            &FLASHBACK_INSTANT,
            &COUNTER_ANY,
            &COUNTER_INSTANT_OR_SORCERY,
            &BOUNCE_ANY,
            &BOTTOM_ANY,
            &COUNTER_BY_WAY,
            &RECYCLER,
            &FILLER,
        ])
    })
}

/// A game in the first player's precombat main phase with priority: `stager` in their hand, `card` the only card of their
/// graveyard, and `Leave Recycler` on the battlefield if `recycler`: the game, the stager and the recycler.
fn setup(stager: &str, card: &str, recycler: bool) -> (RulesGame, ObjectId, Option<ObjectId>) {
    let cards = table();
    let (stager, card, filler) = (cards.find(stager).unwrap(), cards.find(card).unwrap(), cards.find("Leave Filler").unwrap());
    let mut game = RulesGame::new(cards, &Setup { decks: vec![vec![stager, card, filler, filler, filler, filler, filler], vec![filler; 7]], seed: 1, ..Setup::default() });
    let (stager_id, card_id) = (game.state.zone(Zone::hand(ME))[0], game.state.zone(Zone::hand(ME))[1]);
    assert!(game.state.zone_remove(Zone::hand(ME), card_id));
    game.state.update_obj(card_id, |o| o.zone = Zone::graveyard(ME));
    game.state.zone_push(Zone::graveyard(ME), card_id);
    let recycler = recycler.then(|| {
        let id = game.state.push_obj(Obj::new(cards.find("Leave Recycler").unwrap(), ME, Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        id
    });
    game.state.update_turn(|t| {
        t.number = 1;
        t.active = ME;
        t.priority = ME;
        t.set_step(Step::PrecombatMain);
        t.began = true;
    });
    game.state.frame_push(Frame::new(FrameKind::Priority));
    (game, stager_id, recycler)
}

/// Casts the stager and plays on until the stack is empty, confirming payments and passing. Each order of replacement effects
/// asked is answered by `order`, given the sources offered; what was asked is returned.
fn play(game: &mut RulesGame, stager: ObjectId, order: impl Fn(&[ObjectId]) -> usize) -> Vec<Vec<ObjectId>> {
    game.run();
    let cast = match &game.pending().expect("priority").kind {
        RequestKind::Priority { options } => options.iter().find(|o| o.kind == ActionKind::CastSpell && o.source == Some(stager)).cloned().expect("the stager"),
        other => panic!("expected priority, got {other:?}"),
    };
    let request = game.pending().unwrap().clone();
    game.answer(request.id, request.player, Answer::Act(Action::of(&cast))).unwrap();
    let mut asked = Vec::new();
    for _ in 0..32 {
        game.run();
        let cast = game.log.iter().any(|r| matches!(r.event, Event::SpellCast { player: ME, .. }));
        if game.state.stack.is_empty() && cast {
            return asked;
        }
        let request = game.pending().cloned().expect("asked before the stack is empty");
        let answer = match &request.kind {
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { options, .. } => {
                let confirm = options
                    .iter()
                    .find(|o| o.kind == ActionKind::AssignMana)
                    .or_else(|| options.iter().find(|o| o.kind == ActionKind::ConfirmCasting))
                    .unwrap_or_else(|| panic!("unexpected payment options: {options:?}"));
                Answer::Act(Action::of(confirm))
            }
            // The cost: flashback's, where it is offered.
            RequestKind::ChooseModes { modes, eligible, .. } => {
                let way = eligible.iter().copied().find(|&i| matches!(modes[i as usize], Label::Way(..)));
                Answer::Modes(vec![way.unwrap_or(eligible[0])])
            }
            RequestKind::ChooseReplacement { sources, .. } => {
                assert_eq!(request.player, ME, "the spell's controller orders them (CR 616.1)");
                asked.push(sources.clone());
                Answer::Index(order(sources) as u8)
            }
            other => panic!("unexpected request: {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the stack did not empty");
}

/// The zone the one card named `card` is in: its owner's graveyard, exile, hand or library, or the battlefield.
fn where_is(game: &RulesGame, card: &str) -> ZoneKind {
    let card = table().find(card).unwrap();
    let zones = [Zone::graveyard(ME), Zone::exile(ME), Zone::hand(ME), Zone::library(ME), Zone::BATTLEFIELD];
    zones.into_iter().find(|&zone| game.state.zone(zone).iter().any(|&id| game.state.obj(id).card == card)).expect("somewhere").kind()
}

/// "If that spell would be put into your graveyard, exile it instead" names no type: a permanent spell cast this way that is
/// countered is exiled (CR 614.1a) - the creature face Dreadhorde Arcanist may cast, an artifact Jace's permission may.
#[test]
fn a_spell_of_any_type_cast_with_exile_instead_and_countered_is_exiled() {
    let (mut game, stager, _) = setup("Leave Counter Any", "Leave Artifact", false);
    play(&mut game, stager, |_| 0);
    assert_eq!(where_is(&game, "Leave Artifact"), ZoneKind::Exile);
}

/// "If an instant or sorcery spell cast this way would be put into your graveyard" (Bilbo's) does name types: an artifact
/// spell countered goes to the graveyard, an instant is exiled.
#[test]
fn only_an_instant_or_sorcery_is_exiled_where_the_text_says_so() {
    let (mut game, stager, _) = setup("Leave Counter Instant Or Sorcery", "Leave Artifact", false);
    play(&mut game, stager, |_| 0);
    assert_eq!(where_is(&game, "Leave Artifact"), ZoneKind::Graveyard);
    let (mut game, stager, _) = setup("Leave Counter Instant Or Sorcery", "Leave Instant", false);
    play(&mut game, stager, |_| 0);
    assert_eq!(where_is(&game, "Leave Instant"), ZoneKind::Exile);
}

/// It replaces the move to the graveyard alone: a spell returned to its owner's hand, or put into a library, goes there (Jace,
/// Telepath Unbound's rulings), where flashback's "anywhere else" exiles it (CR 702.34a).
#[test]
fn exile_instead_of_the_graveyard_leaves_a_return_to_hand_or_library_alone() {
    let (mut game, stager, _) = setup("Leave Bounce Any", "Leave Instant", false);
    play(&mut game, stager, |_| 0);
    assert_eq!(where_is(&game, "Leave Instant"), ZoneKind::Hand);
    let (mut game, stager, _) = setup("Leave Bottom Any", "Leave Instant", false);
    play(&mut game, stager, |_| 0);
    assert_eq!(where_is(&game, "Leave Instant"), ZoneKind::Library);
    assert_eq!(game.state.zone(Zone::library(ME)).first().map(|&id| game.state.obj(id).card), table().find("Leave Instant"), "on the bottom");
}

/// CR 616.1: the spell's own "exile it instead" is ordered with the other replacement effects that would change its move to
/// the graveyard, by the spell's controller. Taken first, it is exiled; the other first, it goes to its library, and the
/// spell's own no longer applies (CR 616.1f).
#[test]
fn exile_instead_is_ordered_with_the_other_replacements_by_the_spells_controller() {
    for own_first in [true, false] {
        let (mut game, stager, recycler) = setup("Leave Counter Any", "Leave Instant", true);
        let asked = play(&mut game, stager, |sources| sources.iter().position(|&s| (Some(s) != recycler) == own_first).unwrap());
        assert_eq!(asked.len(), 1, "asked once: {asked:?}");
        assert_eq!(asked[0].len(), 2);
        let expected = if own_first { ZoneKind::Exile } else { ZoneKind::Library };
        assert_eq!(where_is(&game, "Leave Instant"), expected, "own first: {own_first}");
    }
}

/// Flashback's is ordered with them too (CR 616.1), and as it replaces a move anywhere but exile, it applies again once the
/// other has sent the card to its library (CR 616.1f): exiled either way.
#[test]
fn flashback_is_ordered_with_the_other_replacements_and_applies_after_them() {
    for own_first in [true, false] {
        let (mut game, stager, recycler) = setup("Leave Counter By Way", "Leave Flashback Instant", true);
        let asked = play(&mut game, stager, |sources| sources.iter().position(|&s| (Some(s) != recycler) == own_first).unwrap());
        assert_eq!(asked.len(), 1, "asked once: {asked:?}");
        assert_eq!(where_is(&game, "Leave Flashback Instant"), ZoneKind::Exile, "own first: {own_first}");
    }
}
