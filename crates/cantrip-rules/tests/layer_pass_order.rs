//! The layers' order is a function of the position (CR 613.8): an object's characteristics do not depend on which object
//! was asked about first, and a dependency is judged over every object an effect could apply to - in a graveyard too, for
//! an effect whose words reach every zone. On test-only cards (the reviewers' probes of the dependency system).

use std::sync::OnceLock;

use cantrip_api::*;
use cantrip_rules::{Game, Setup};

fn lands(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Land)
}

fn add_creature(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
    c.types = c.types.with(CardType::Creature);
}

fn enchantments(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Enchantment)
}

fn only_land(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
    c.types = TypeSet::of(&[CardType::Land]);
}

fn artifacts(_: &dyn LayerQuery, _: ObjectId, _: ObjectId, c: &Chars) -> bool {
    c.is(CardType::Artifact)
}

fn add_enchantment(_: &dyn LayerQuery, _: ObjectId, c: &mut Chars) {
    c.types = c.types.with(CardType::Enchantment);
}

/// "Each land is a creature in addition to its other types", in every zone.
static ANIMATOR: CardDef = CardDef {
    name: "Probe Animator",
    types: TypeSet::of(&[CardType::Instant]),
    statics: &[static_ability(Layer::Type, lands, add_creature, "Each land is a creature in addition to its other types.").everywhere()],
    ..CardDef::BLANK
};

/// "Each enchantment is a land" - its card types become Land alone - in every zone.
static GROUNDSKEEPER: CardDef = CardDef {
    name: "Probe Groundskeeper",
    types: TypeSet::of(&[CardType::Instant]),
    statics: &[static_ability(Layer::Type, enchantments, only_land, "Each enchantment is a land.").everywhere()],
    ..CardDef::BLANK
};

/// "Each artifact is an enchantment in addition to its other types", in every zone.
static ENCHANTER: CardDef = CardDef {
    name: "Probe Enchanter",
    types: TypeSet::of(&[CardType::Instant]),
    statics: &[static_ability(Layer::Type, artifacts, add_enchantment, "Each artifact is an enchantment in addition.").everywhere()],
    ..CardDef::BLANK
};

static GROVE: CardDef = CardDef { name: "Probe Grove", has_cost: false, types: TypeSet::of(&[CardType::Enchantment, CardType::Land]), ..CardDef::BLANK };
static CHARM: CardDef = CardDef { name: "Probe Charm", types: TypeSet::of(&[CardType::Enchantment]), ..CardDef::BLANK };
static TRINKET: CardDef = CardDef { name: "Probe Trinket", types: TypeSet::of(&[CardType::Artifact]), ..CardDef::BLANK };
static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static T: OnceLock<CardTable> = OnceLock::new();
    T.get_or_init(|| CardTable::new([&ANIMATOR, &GROUNDSKEEPER, &ENCHANTER, &GROVE, &CHARM, &TRINKET, &FILLER]))
}

/// A game with the named cards where named, in timestamp order as named, and each object's characteristics, asked in the
/// order `asked` gives (indices into `cards`).
fn board(cards: &[(&str, Zone)], asked: &[usize]) -> Vec<Option<Chars>> {
    let filler = table().find("Filler").unwrap();
    let mut game = Game::new(table(), &Setup { decks: vec![vec![filler; 30], vec![filler; 30]], seed: 1, shuffle: false, ..Setup::default() });
    let mut ids = Vec::new();
    for &(name, zone) in cards {
        let mut obj = Obj::new(table().find(name).unwrap(), PlayerId(0), zone);
        obj.timestamp = game.state.timestamp();
        let id = game.state.push_obj(obj);
        game.state.zone_push(zone, id);
        ids.push(id);
    }
    game.state.see_statics();
    let mut found = vec![None; cards.len()];
    for &k in asked {
        found[k] = Some(game.chars(ids[k]));
    }
    found
}

/// CR 613.8a: the Groundskeeper makes the Charm in a graveyard a land, which the Animator then applies to: the Animator
/// depends on it and applies after it, though it is older. The Grove and the Charm are Land Creatures, whichever is asked
/// first.
#[test]
fn a_dependency_through_a_card_in_a_graveyard_is_judged_whoever_asks_first() {
    let cards = [
        ("Probe Animator", Zone::BATTLEFIELD),
        ("Probe Groundskeeper", Zone::BATTLEFIELD),
        ("Probe Grove", Zone::BATTLEFIELD),
        ("Probe Charm", Zone::graveyard(PlayerId(0))),
    ];
    for asked in [[2, 3], [3, 2]] {
        let found = board(&cards, &asked);
        for k in [2, 3] {
            let c = found[k].expect("asked");
            assert!(c.is(CardType::Land) && c.is(CardType::Creature), "{asked:?}: {k} is a Land Creature");
            assert!(!c.is(CardType::Enchantment), "{asked:?}: {k} is no enchantment");
        }
    }
}

/// CR 613.8a-c, judged on the game at each step: the Animator, the oldest, depends on nothing (the Grove is a land either
/// way, and nothing else becomes one before it applies) and applies first; the Groundskeeper waits for the Enchanter, which
/// makes the Trinket in a graveyard an enchantment; then the Groundskeeper makes the Grove and the Trinket lands alone. The
/// same whichever is asked first.
#[test]
fn the_order_is_the_same_whichever_object_is_asked_first() {
    let cards = [
        ("Probe Animator", Zone::BATTLEFIELD),
        ("Probe Groundskeeper", Zone::BATTLEFIELD),
        ("Probe Enchanter", Zone::BATTLEFIELD),
        ("Probe Grove", Zone::BATTLEFIELD),
        ("Probe Trinket", Zone::graveyard(PlayerId(0))),
    ];
    let first = board(&cards, &[3, 4]);
    let second = board(&cards, &[4, 3]);
    assert_eq!(first, second, "characteristics are a function of the position");
    for k in [3, 4] {
        assert_eq!(first[k].expect("asked").types, TypeSet::of(&[CardType::Land]), "{k} is a Land alone");
    }
}
