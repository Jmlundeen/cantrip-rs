//! "Enchanted land has indestructible and can't be enchanted by other Auras." (Consecrate Land) and "Enchanted creature can't be
//! the target of spells and can't be enchanted by other Auras." (Anti-Magic Aura): `TargetingRuleKind::NoOtherAuras`, a rule of
//! the Aura's (CR 303.4: "Other effects can limit what a permanent can be enchanted by"). Another Aura spell may target the
//! permanent (CR 303.4a: its enchant ability alone defines its targets) and, resolving, is put into its owner's graveyard
//! instead of entering (CR 303.4i); another Aura can't enter attached to it (CR 303.4f, 303.4i) nor move onto it (CR 303.4j), and one
//! already attached is put into its owner's graveyard (CR 303.4c, 704.5m; Consecrate Land's 2006 ruling). The granting Aura
//! itself stays; a second copy is another Aura, so it is refused, and two attached at once remove each other. The rule follows
//! the Aura where it moves, ends as it leaves, and ends with the Aura's abilities (CR 613.1f) - not with the permanent's, whose
//! ability it is not. Over test cards shaped like Anti-Magic Aura.

use crate::{Cx, Game, Setup};
use cantrip_api::*;
use cantrip_mechanics::attach::ENCHANTED_NO_OTHER_AURAS;
use cantrip_state::{Frame, FrameKind};

const YOU: PlayerId = PlayerId(0);

static LAND: CardDef = CardDef { name: "Sole Land", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

static BEAR: CardDef = CardDef {
    name: "Sole Bear",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

static SORCERY: CardDef = CardDef { name: "Sole Sorcery", types: TypeSet::of(&[CardType::Sorcery]), ..CardDef::BLANK };

fn creature_on_the_battlefield(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).is(CardType::Creature))
}

/// An "enchant creature" Aura with `rules`.
const fn aura(name: &'static str, rules: &'static [TargetingRule]) -> CardDef {
    CardDef {
        name,
        colors: Some(ColorSet::of(Color::Blue)),
        types: TypeSet::of(&[CardType::Enchantment]),
        subtypes: SubtypeSet::of(&[Subtype::Aura]),
        enchant: Some(target(creature_on_the_battlefield)),
        targeting_rules: rules,
        ..CardDef::BLANK
    }
}

/// "Enchanted creature can't be enchanted by other Auras."
static SOLE_AURA: CardDef = aura("Sole Aura", &[ENCHANTED_NO_OTHER_AURAS]);
static PLAIN_AURA: CardDef = aura("Plain Aura", &[]);

static EQUIPMENT: CardDef = CardDef {
    name: "Sole Equipment",
    types: TypeSet::of(&[CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Equipment]),
    ..CardDef::BLANK
};

fn game() -> Game {
    let table: &'static CardTable =
        Box::leak(Box::new(CardTable::new([&LAND, &BEAR, &SORCERY, &SOLE_AURA, &PLAIN_AURA, &EQUIPMENT])));
    let deck = || vec![table.id_of(&LAND); 20];
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

fn put(g: &mut Game, def: &'static CardDef, zone: Zone) -> ObjectId {
    let id = g.state.push_obj(Obj::new(g.cards.id_of(def), YOU, zone));
    let timestamp = g.state.timestamp();
    g.state.update_obj(id, |o| o.timestamp = timestamp);
    g.state.zone_push(zone, id);
    g.state.see_statics();
    id
}

/// `def` on the battlefield, attached to `host` however it got there.
fn attached(g: &mut Game, def: &'static CardDef, host: ObjectId) -> ObjectId {
    let id = put(g, def, Zone::BATTLEFIELD);
    g.state.update_obj(id, |o| o.attached_to = Some(host));
    id
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

/// The state-based actions performed in a settling frame of the game's own, as the rules perform them.
fn state_based_actions(g: &mut Game) {
    g.state.frame_push(Frame::new(FrameKind::Settle));
    let frame = g.state.agenda.len() - 1;
    let mut cx = Cx { g, frame, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None };
    crate::settle::state_based_actions_alone(&mut cx).expect("nothing is asked");
}

fn stays(g: &Game, id: ObjectId, host: ObjectId) -> bool {
    g.state.obj(id).zone.is(ZoneKind::Battlefield) && g.state.obj(id).attached_to == Some(host)
}

fn went_to_the_graveyard(g: &Game, id: ObjectId) -> bool {
    g.state.obj(id).became.is_some_and(|new| g.state.obj(new).zone.is(ZoneKind::Graveyard))
}

/// Whether the spell `spell` on the stack could target `what` (CR 601.2c, 608.2b).
fn spell_can_target(g: &Game, spell: ObjectId, what: ObjectId) -> bool {
    let asker = Asker { source: spell, controller: YOU, by: None, spell: true };
    crate::layers::can_target(g.cards, &g.state, &g.log, &asker, Target::Object(what))
}

/// Whether another Aura could be attached to `host` now: by an effect that attaches (CR 303.4j, `Game::can_attach`).
fn can_attach(g: &mut Game, aura: ObjectId, host: ObjectId) -> bool {
    cantrip_api::Game::can_attach(&cx(g), aura, host)
}

/// "Can't be enchanted by" limits what may enchant, not what may be targeted: an Aura spell's legal targets are its enchant
/// ability's alone (CR 303.4a, 115.1b, 702.5a), so another Aura spell - a second copy too - may target the enchanted creature.
#[test]
fn another_aura_spell_may_target_the_permanent() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    attached(&mut g, &SOLE_AURA, bear);
    let spell = put(&mut g, &PLAIN_AURA, Zone::STACK);
    let sorcery = put(&mut g, &SORCERY, Zone::STACK);
    assert!(spell_can_target(&g, spell, bear), "an Aura spell's targets are its enchant ability's (CR 303.4a)");
    assert!(spell_can_target(&g, sorcery, bear));
    let copy = put(&mut g, &SOLE_AURA, Zone::STACK);
    assert!(spell_can_target(&g, copy, bear), "a second copy may target it too");
}

/// Another Aura can't enter attached to it, named (CR 303.4i: it stays where it is) or chosen (CR 303.4f).
#[test]
fn another_aura_can_not_enter_attached_to_the_permanent() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let free = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    attached(&mut g, &SOLE_AURA, bear);
    let card = put(&mut g, &PLAIN_AURA, Zone::graveyard(YOU));
    assert!(!cantrip_api::Game::could_enter_attached(&mut cx(&mut g), card, YOU, bear));
    assert!(cantrip_api::Game::could_enter_attached(&mut cx(&mut g), card, YOU, free));
    let entered = cantrip_api::Game::put_onto_battlefield_attached(&mut cx(&mut g), card, YOU, bear).expect("nothing is asked");
    assert_eq!(entered, None, "it can't enter attached (CR 303.4i)");
    assert!(g.state.obj(card).zone.is(ZoneKind::Graveyard), "it stays where it is");
    let cx = cx(&mut g);
    let chars = cx.chars(card);
    let hosts = crate::attach::aura_hosts(&cx, card, &chars, YOU, &[]);
    assert!(!hosts.contains(&Target::Object(bear)) && hosts.contains(&Target::Object(free)), "it can't be chosen (CR 303.4f)");
}

/// Another Aura on the battlefield can't be moved onto it (CR 303.4j: the Aura doesn't move); an Equipment, no Aura, can.
#[test]
fn another_aura_can_not_move_onto_the_permanent() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let free = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    attached(&mut g, &SOLE_AURA, bear);
    let other = attached(&mut g, &PLAIN_AURA, free);
    let equipment = put(&mut g, &EQUIPMENT, Zone::BATTLEFIELD);
    assert!(!can_attach(&mut g, other, bear));
    assert!(!cantrip_api::Game::attach(&mut cx(&mut g), other, bear), "it doesn't move");
    assert!(stays(&g, other, free));
    assert!(can_attach(&mut g, equipment, bear), "an Equipment is no Aura");
}

/// An Aura already attached is put into its owner's graveyard (CR 704.5m; Consecrate Land's ruling); the granting Aura stays.
#[test]
fn an_aura_already_attached_goes_and_the_granting_aura_stays() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let other = attached(&mut g, &PLAIN_AURA, bear);
    let sole = attached(&mut g, &SOLE_AURA, bear);
    assert!(crate::attach::host_allows(&cx(&mut g), sole, bear, false), "the granting Aura is no other Aura");
    state_based_actions(&mut g);
    assert!(stays(&g, sole, bear));
    assert!(went_to_the_graveyard(&g, other));
}

/// A second copy is "another Aura": it can't be attached, and two attached at once each make the other illegal - both go
/// (CR 704.5m, performed simultaneously, CR 704.3).
#[test]
fn a_second_copy_is_refused_and_two_attached_remove_each_other() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let free = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    attached(&mut g, &SOLE_AURA, bear);
    let second = attached(&mut g, &SOLE_AURA, free);
    assert!(!can_attach(&mut g, second, bear), "the second copy can't be attached");
    state_based_actions(&mut g);
    assert!(stays(&g, second, free), "each restricts only its own host");

    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let first = attached(&mut g, &SOLE_AURA, bear);
    let second = attached(&mut g, &SOLE_AURA, bear);
    state_based_actions(&mut g);
    assert!(went_to_the_graveyard(&g, first) && went_to_the_graveyard(&g, second), "both go");
}

/// The rule follows the granting Aura: moved to another creature, it restricts that one and frees the first; gone from the
/// battlefield, it restricts none.
#[test]
fn the_restriction_follows_the_aura_and_ends_as_it_leaves() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let free = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let sole = attached(&mut g, &SOLE_AURA, bear);
    let loose = put(&mut g, &PLAIN_AURA, Zone::BATTLEFIELD);
    assert!(!can_attach(&mut g, loose, bear));
    assert!(cantrip_api::Game::attach(&mut cx(&mut g), sole, free), "the granting Aura moves");
    assert!(can_attach(&mut g, loose, bear), "the first creature is free");
    assert!(!can_attach(&mut g, loose, free), "the second is not");
    // Unattached, it is put into its owner's graveyard (CR 704.5m), and so is the loose Aura.
    cantrip_api::Game::unattach(&mut cx(&mut g), sole);
    state_based_actions(&mut g);
    assert!(went_to_the_graveyard(&g, sole));
    let loose = put(&mut g, &PLAIN_AURA, Zone::BATTLEFIELD);
    assert!(can_attach(&mut g, loose, free), "it has left the battlefield");
    let card = put(&mut g, &PLAIN_AURA, Zone::graveyard(YOU));
    assert!(cantrip_api::Game::could_enter_attached(&mut cx(&mut g), card, YOU, free));
}

/// A rule of the Aura's, not an ability of the creature (CR 113.1a): the creature losing all its abilities is still under it;
/// the Aura losing all its abilities ends it (CR 613.1f).
#[test]
fn the_aura_losing_its_abilities_ends_it_and_the_creature_losing_its_abilities_does_not() {
    let mut g = game();
    let bear = put(&mut g, &BEAR, Zone::BATTLEFIELD);
    let sole = attached(&mut g, &SOLE_AURA, bear);
    let loose = put(&mut g, &PLAIN_AURA, Zone::BATTLEFIELD);
    let lose_all = |g: &mut Game, affected: ObjectId| {
        let timestamp = g.state.timestamp();
        g.state.effect_push(ContinuousEffect {
            source: affected,
            controller: YOU,
            timestamp,
            affected,
            kind: EffectKind::LoseAllAbilities,
            duration: Duration::Indefinite,
            words: Words::NONE,
        });
    };
    lose_all(&mut g, bear);
    assert!(!can_attach(&mut g, loose, bear), "the creature's abilities are not the rule");
    lose_all(&mut g, sole);
    assert!(can_attach(&mut g, loose, bear), "the Aura has no static abilities (CR 613.1f)");
}
