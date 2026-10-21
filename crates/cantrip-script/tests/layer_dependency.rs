//! A dependency through another object (CR 613.8a): "applying the other would change ... what it applies to" - what an effect
//! applies to is a set of objects, so one effect can depend on another though the object being worked out shows nothing of
//! it. On a table of test-only cards, as no two printed cards in the engine meet that way:
//!
//! - Probe Animator (older): "Each land is a creature in addition to its other types."
//! - Probe Groundskeeper (newer): "Each enchantment is a land" - its card types become Land alone.
//! - Probe Grove is an Enchantment Land, Probe Charm an Enchantment.
//!
//! The Groundskeeper changes what the Animator applies to (the Charm becomes a land), so the Animator depends on it and applies
//! after it though it is older (CR 613.8b). The Grove is a land either way, so on its own it shows no dependency; but the two
//! effects don't commute on it, and in dependency order it ends a Land Creature, where in timestamp order the Groundskeeper
//! would have left it a Land alone.

use std::sync::OnceLock;

use cantrip_engine::*;

fn lands(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Land)
}

fn add_creature(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
}

fn enchantments(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Enchantment)
}

fn only_land(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = TypeSet::of(&[CardType::Land]);
}

static ANIMATOR: CardDef = CardDef {
    name: "Probe Animator",
    types: TypeSet::of(&[CardType::Artifact]),
    statics: &[static_ability(Layer::Type, lands, add_creature, "Each land is a creature in addition to its other types.")],
    ..CardDef::BLANK
};

static GROUNDSKEEPER: CardDef = CardDef {
    name: "Probe Groundskeeper",
    types: TypeSet::of(&[CardType::Artifact]),
    statics: &[static_ability(Layer::Type, enchantments, only_land, "Each enchantment is a land.")],
    ..CardDef::BLANK
};

static GROVE: CardDef =
    CardDef { name: "Probe Grove", has_cost: false, types: TypeSet::of(&[CardType::Enchantment, CardType::Land]), ..CardDef::BLANK };

static CHARM: CardDef = CardDef { name: "Probe Charm", types: TypeSet::of(&[CardType::Enchantment]), ..CardDef::BLANK };

fn artifacts(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Artifact)
}

fn artifact_creatures(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Artifact) && chars.is(CardType::Creature)
}

fn golem(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(Subtype::Golem);
}

/// Whether the source's controller controls an artifact creature other than `id`, as the layer stands (read through the
/// view: another object).
fn while_you_control_another_artifact_creature(view: &dyn LayerQuery, source: ObjectId, id: ObjectId, chars: &Chars) -> bool {
    let you = view.state().obj(source).controller;
    chars.is(CardType::Artifact)
        && view.state().battlefield().iter().any(|&other| {
            other != id && view.state().obj(other).controller == you && {
                let c = view.chars(other);
                c.is(CardType::Artifact) && c.is(CardType::Creature)
            }
        })
}

fn add_creature_to(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.types = chars.types.with(CardType::Creature);
}

/// "Each artifact that's a creature is a Golem in addition to its other types."
static FOUNDRY: CardDef = CardDef {
    name: "Probe Foundry",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Type, artifact_creatures, golem, "Each artifact that's a creature is a Golem in addition to its other types.")],
    ..CardDef::BLANK
};

/// "As long as you control an artifact creature, each other artifact you control is a Golem in addition to its other types":
/// whether it applies to an artifact depends on another object.
static FORGE: CardDef = CardDef {
    name: "Probe Forge",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(
        Layer::Type,
        while_you_control_another_artifact_creature,
        golem,
        "As long as you control an artifact creature, each other artifact you control is a Golem in addition to its other types.",
    )],
    ..CardDef::BLANK
};

/// "Each artifact is a creature in addition to its other types."
static WAKER: CardDef = CardDef {
    name: "Probe Waker",
    types: TypeSet::of(&[CardType::Enchantment]),
    statics: &[static_ability(Layer::Type, artifacts, add_creature_to, "Each artifact is a creature in addition to its other types.")],
    ..CardDef::BLANK
};

static TRINKET: CardDef = CardDef { name: "Probe Trinket", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&ANIMATOR, &GROUNDSKEEPER, &GROVE, &CHARM, &FOUNDRY, &FORGE, &WAKER, &TRINKET, &FILLER]))
}

/// The named permanents on the battlefield, in timestamp order as named.
fn board(names: &[&str]) -> (Game, Vec<ObjectId>) {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, ..Setup::default() });
    let ids = names
        .iter()
        .map(|name| {
            let mut obj = Obj::new(table().find(name).unwrap(), PlayerId(0), Zone::BATTLEFIELD);
            obj.timestamp = game.state.timestamp();
            let id = game.state.push_obj(obj);
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        })
        .collect();
    game.state.see_statics();
    (game, ids)
}

/// CR 613.8a-b: with the Charm in play the Animator depends on the Groundskeeper, and the Grove is a Land Creature.
#[test]
fn an_effect_that_depends_on_another_through_another_object_waits_for_it() {
    let (game, ids) = board(&["Probe Animator", "Probe Groundskeeper", "Probe Grove", "Probe Charm"]);
    let grove = game.chars(ids[2]);
    assert!(grove.is(CardType::Land) && grove.is(CardType::Creature), "the Grove is a Land Creature");
    assert!(!grove.is(CardType::Enchantment));
    let charm = game.chars(ids[3]);
    assert!(charm.is(CardType::Land) && charm.is(CardType::Creature), "the Charm is a Land Creature");
}

/// Without the Charm the Groundskeeper changes nothing the Animator applies to: no dependency, timestamp order (CR 613.7), and
/// the Groundskeeper, newer, leaves the Grove a Land alone.
#[test]
fn without_the_other_object_the_effects_apply_in_timestamp_order() {
    let (game, ids) = board(&["Probe Animator", "Probe Groundskeeper", "Probe Grove"]);
    let grove = game.chars(ids[2]);
    assert!(grove.is(CardType::Land) && !grove.is(CardType::Creature), "the Grove is a Land alone");
}

/// CR 613.8a(b): the Waker changes what the older Foundry applies to (it makes the Trinket an artifact creature), so the
/// Foundry depends on the Waker and goes after it: the Trinket is a Golem artifact creature. In timestamp order it would be
/// no Golem.
#[test]
fn an_effect_whose_reach_another_changes_waits_for_it() {
    let (game, ids) = board(&["Probe Foundry", "Probe Waker", "Probe Trinket"]);
    let trinket = game.chars(ids[2]);
    assert!(trinket.is(CardType::Creature) && trinket.has_subtype(Subtype::Golem), "the Trinket is a Golem creature");
}

/// CR 613.8a(b), through another object: the Forge applies to the Charm-less board's second Trinket only while another
/// artifact creature is controlled, which the Waker makes the first Trinket; so the older Forge depends on the Waker, and both
/// Trinkets are Golems.
#[test]
fn an_effect_whose_condition_another_object_meets_once_another_applies_waits_for_it() {
    let (game, ids) = board(&["Probe Forge", "Probe Waker", "Probe Trinket", "Probe Trinket"]);
    for &id in &ids[2..] {
        let trinket = game.chars(id);
        assert!(trinket.is(CardType::Creature) && trinket.has_subtype(Subtype::Golem), "each Trinket is a Golem creature");
    }
}

/// The Charm is made a land by an older resolved effect, and the Grove is worked out alone or after the Charm: a permanent's
/// characteristics must not depend on which object was asked first in a position (the review of the pair judgment).
fn probe(charm_first: bool) -> Chars {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, ..Setup::default() });
    let early = game.state.timestamp();
    let ids: Vec<ObjectId> = ["Probe Animator", "Probe Groundskeeper", "Probe Grove", "Probe Charm"]
        .iter()
        .map(|name| {
            let mut obj = Obj::new(table().find(name).unwrap(), PlayerId(0), Zone::BATTLEFIELD);
            obj.timestamp = game.state.timestamp();
            let id = game.state.push_obj(obj);
            game.state.zone_push(Zone::BATTLEFIELD, id);
            id
        })
        .collect();
    game.state.effect_push(ContinuousEffect {
        source: ids[3],
        controller: PlayerId(0),
        timestamp: early,
        affected: ids[3],
        kind: EffectKind::AddTypes(TypeSet::of(&[CardType::Land])),
        duration: Duration::Indefinite,
    });
    game.state.see_statics();
    if charm_first {
        let _ = game.chars(ids[3]);
    }
    game.chars(ids[2])
}

#[test]
fn the_groves_characteristics_do_not_depend_on_who_was_asked_first() {
    let alone = probe(false);
    let after = probe(true);
    assert_eq!(alone, after, "characteristics are a function of the position");
    // The Charm is a land already, so the Groundskeeper changes nothing the Animator applies to, nor what it does: no
    // dependency, timestamp order (CR 613.7), and the newer Groundskeeper leaves the Grove a Land alone.
    assert!(alone.is(CardType::Land) && !alone.is(CardType::Creature));
}
