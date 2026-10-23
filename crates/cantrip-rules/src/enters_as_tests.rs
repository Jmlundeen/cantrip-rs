//! An effect that puts a card onto the battlefield as something it says (`Game::put_onto_battlefield_as`, `EntersAs`): the
//! card enters as that (CR 614.12) - an Aura enchantment with the enchant ability the effect gives it, never a creature - and
//! as an Aura chooses what it enchants as it enters, without targeting (CR 303.4f), stays where it is with nothing to enchant
//! (CR 303.4g), and is put into its owner's graveyard once what it enchants is no longer legal (CR 303.4c, 704.5m). Over test
//! cards shaped like Old-Growth Troll's "return it to the battlefield. It's an Aura enchantment with enchant Forest you
//! control and '...'".

use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);
const THEM: PlayerId = PlayerId(1);

static FILLER: CardDef = CardDef { name: "Enters As Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

static FOREST: CardDef = CardDef {
    name: "Enters As Forest",
    has_cost: false,
    types: TypeSet::of(&[CardType::Land]),
    subtypes: SubtypeSet::of(&[Subtype::Forest]),
    ..CardDef::BLANK
};

/// "Enchant Forest you control" (CR 702.5a): judged for the Aura's controller.
fn forest_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).has_subtype(Subtype::Forest) && q.controller(id) == asker.controller)
}

fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

fn never(_: &dyn Query, _: ObjectId, _: &Event) -> Option<Fired> {
    None
}

/// A 4/4 Troll Warrior with trample whose effect returns it as "an Aura enchantment with enchant Forest you control and
/// '{T}: Add {G}{G}.'"
static TROLL: CardDef = CardDef {
    name: "Enters As Troll",
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Troll, Subtype::Warrior]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Trample]),
    abilities: &[triggered(cantrip_api::when(EventMask::ZONE_CHANGE, never), nothing, "When this creature dies, return it.")],
    becomes_aura_enchant: Some(target(forest_you_control)),
    delayed: &[mana_ability(TAP, add(Mana::G), "{T}: Add {G}{G}.")],
    ..CardDef::BLANK
};

static AS_AURA: EntersAs = EntersAs { types: Some(TypeSet::of(&[CardType::Enchantment])), aura: true, abilities: &[0], ..EntersAs::NONE };

/// "Whenever a creature enters, ...": what would see the returning card if it entered as a creature.
fn a_creature_enters(q: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. } if to.is(ZoneKind::Battlefield) && q.chars(*new).is(CardType::Creature) => Fired::PLAIN,
        _ => None,
    }
}

static WATCHER: CardDef = CardDef {
    name: "Enters As Watcher",
    types: TypeSet::of(&[CardType::Enchantment]),
    abilities: &[triggered(cantrip_api::when(EventMask::ZONE_CHANGE, a_creature_enters), nothing, "Whenever a creature enters, nothing.")],
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&FILLER, &FOREST, &TROLL, &WATCHER])));
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
    g.state.update_player(YOU, |p| p.turn_began = 1);
    g
}

fn put(g: &mut Game, def: &'static CardDef, owner: PlayerId, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), owner, zone));
    let timestamp = g.state.timestamp();
    g.state.update_obj(id, |o| o.timestamp = timestamp);
    g.state.zone_push(zone, id);
    g.state.see_statics();
    id
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
}

/// The Troll's ability returning the Troll card `card` from YOU's graveyard as the Aura.
fn return_as_aura(g: &mut Game, card: ObjectId) -> Option<ObjectId> {
    let ability = AbilityRef { card: g.cards.id_of(&TROLL), index: 0 };
    let item = crate::actions::stack_item(StackKind::Triggered, card, ability, YOU, &[]);
    cantrip_api::Game::put_onto_battlefield_as(&mut cx(g), card, YOU, &item, &AS_AURA).expect("one host: nothing to ask")
}

/// It enters attached to the one Forest its controller controls, as an Aura enchantment that is no creature - nothing that
/// watches creatures enter sees it - with its own trample and the ability it is given.
#[test]
fn it_enters_attached_to_a_legal_host_as_a_non_creature_aura() {
    let mut g = game();
    put(&mut g, &WATCHER, YOU, Zone::BATTLEFIELD);
    let forest = put(&mut g, &FOREST, YOU, Zone::BATTLEFIELD);
    put(&mut g, &FOREST, THEM, Zone::BATTLEFIELD);
    let card = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    let aura = return_as_aura(&mut g, card).expect("it entered");
    let o = *g.state.obj(aura);
    assert!(o.zone.is(ZoneKind::Battlefield));
    assert_eq!(o.attached_to, Some(forest));
    let q: &dyn Query = &cx(&mut g);
    let chars = q.chars(aura);
    assert!(chars.is(CardType::Enchantment) && !chars.is(CardType::Creature));
    assert!(chars.has_subtype(Subtype::Aura) && !chars.has_subtype(Subtype::Troll) && !chars.has_subtype(Subtype::Warrior));
    assert!(chars.has(Keyword::Trample));
    assert!(chars.abilities.contains(&Some(AbilityRef::delayed(g.cards.id_of(&TROLL), 0))));
    assert!(g.state.triggers.is_empty(), "it never entered as a creature");
    assert!(g.log.iter().any(|r| matches!(r.event, Event::Attached { attachment, to: Target::Object(host) } if attachment == aura && host == forest)));

    // The watcher does see a creature enter: the check above is a real one.
    let other = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    cantrip_api::Game::put_onto_battlefield(&mut cx(&mut g), other, YOU, false).expect("nothing to ask");
    assert!(!g.state.triggers.is_empty());
}

/// With no Forest its controller controls, it stays in the graveyard (CR 303.4g): an opponent's Forest is no legal host.
#[test]
fn with_no_legal_host_it_stays_in_the_graveyard() {
    let mut g = game();
    put(&mut g, &FOREST, THEM, Zone::BATTLEFIELD);
    let card = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    assert_eq!(return_as_aura(&mut g, card), None);
    assert!(g.state.obj(card).zone.is(ZoneKind::Graveyard));
    assert!(g.state.obj(card).became.is_none(), "the card did not move");
    assert!(g.state.effects.is_empty(), "nothing was given the card");
}

/// Once the Forest it enchants is no longer one its controller controls, the state-based actions put it into its owner's
/// graveyard (CR 303.4c, 704.5m): the enchant ability it was given goes on judging its host.
#[test]
fn it_falls_off_when_its_host_is_no_longer_legal() {
    let mut g = game();
    let forest = put(&mut g, &FOREST, YOU, Zone::BATTLEFIELD);
    let card = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    let aura = return_as_aura(&mut g, card).expect("it entered");
    assert!(!crate::settle::state_based_actions_alone(&mut cx(&mut g)).unwrap_or(true), "attached legally, it stays");
    assert!(g.state.battlefield().contains(&aura));

    cantrip_api::Game::add_effect(&mut cx(&mut g), forest, forest, EffectKind::Control(THEM), Duration::Indefinite);
    assert!(crate::settle::state_based_actions_alone(&mut cx(&mut g)).unwrap_or(false));
    assert!(!g.state.battlefield().contains(&aura));
    let now = g.state.obj(aura).became.expect("it moved");
    assert_eq!(g.state.obj(now).zone, Zone::graveyard(YOU));
}

/// Its enchant ability is one of its abilities (CR 702.5a, 613.1f): once a later effect takes all its abilities, nothing
/// restricts what it enchants, and a Forest no longer its controller's is not illegal on that ground (CR 303.4c, 704.5m).
#[test]
fn an_aura_that_lost_its_enchant_stays_when_its_host_stops_matching() {
    let mut g = game();
    let forest = put(&mut g, &FOREST, YOU, Zone::BATTLEFIELD);
    let card = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    let aura = return_as_aura(&mut g, card).expect("it entered");
    cantrip_api::Game::add_effect(&mut cx(&mut g), aura, aura, EffectKind::LoseAllAbilities, Duration::Indefinite);
    assert!(!cx(&mut g).chars(aura).abilities().any(|a| a.is_enchant()), "the enchant ability is lost with the rest");
    cantrip_api::Game::add_effect(&mut cx(&mut g), forest, forest, EffectKind::Control(THEM), Duration::Indefinite);
    assert!(!crate::settle::state_based_actions_alone(&mut cx(&mut g)).unwrap_or(true));
    assert!(g.state.battlefield().contains(&aura));
    assert_eq!(g.state.obj(aura).attached_to, Some(forest));
}

/// An Aura that is also a creature can enchant nothing (CR 303.4d): made a creature, it is put into its owner's graveyard.
#[test]
fn an_aura_that_becomes_a_creature_falls_off() {
    let mut g = game();
    put(&mut g, &FOREST, YOU, Zone::BATTLEFIELD);
    let card = put(&mut g, &TROLL, YOU, Zone::graveyard(YOU));
    let aura = return_as_aura(&mut g, card).expect("it entered");
    cantrip_api::Game::add_effect(&mut cx(&mut g), aura, aura, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    assert!(crate::settle::state_based_actions_alone(&mut cx(&mut g)).unwrap_or(false));
    assert!(!g.state.battlefield().contains(&aura));
}
