//! An animation that makes a permanent "a [creature types] artifact creature" without saying it is still what it was
//! replaces its creature types with those named, keeping its card types and its other subtypes (CR 205.1b:
//! `Animation::replaces_creature_types`, Iron Man Armor's "a 0/0 Construct Hero artifact creature"). The Armor has no creature
//! type of its own to lose and no card in content gives a noncreature permanent one, so the replacement is shown here on a
//! test-only Kindred artifact - a noncreature permanent with a creature type - beside the animation that keeps them ("It's
//! still a land", Inkmoth Nexus).

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::{Animation, animate};

fn artifact(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.chars(id).is(CardType::Artifact) && q.obj(id).zone.is(ZoneKind::Battlefield))
}

fn animation(replaces_creature_types: bool) -> Animation<'static> {
    Animation {
        power: 2,
        toughness: 2,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[Subtype::Construct],
        all_creature_types: false,
        replaces_creature_types,
        keywords: Keywords::NONE,
        ability: None,
    }
}

fn the_artifact(item: &StackItem) -> ObjectId {
    match item.targets()[0] {
        Target::Object(id) => id,
        other => panic!("an artifact, not {other:?}"),
    }
}

/// Until end of turn, target artifact becomes a 2/2 Construct artifact creature.
fn replace(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    animate(g, item.object, the_artifact(item), &animation(true), Duration::EndOfTurn);
    Ok(())
}

/// Until end of turn, target artifact becomes a 2/2 Construct artifact creature. It's still what it was.
fn keep(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    animate(g, item.object, the_artifact(item), &animation(false), Duration::EndOfTurn);
    Ok(())
}

static ONE: [TargetSpec; 1] = [target(artifact)];

static REPLACE: CardDef =
    CardDef { name: "Probe Replace", types: TypeSet::of(&[CardType::Instant]), spell: spell(&ONE, replace), ..CardDef::BLANK };
static KEEP: CardDef = CardDef { name: "Probe Keep", types: TypeSet::of(&[CardType::Instant]), spell: spell(&ONE, keep), ..CardDef::BLANK };

/// A Kindred artifact, an Elf and an Equipment: a noncreature permanent with a creature type and another subtype.
static IDOL: CardDef = CardDef {
    name: "Probe Idol",
    types: TypeSet::of(&[CardType::Kindred, CardType::Artifact]),
    subtypes: SubtypeSet::of(&[Subtype::Elf, Subtype::Equipment]),
    ..CardDef::BLANK
};

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&REPLACE, &KEEP, &IDOL, &FILLER]))
}

/// The first seat holds `spell` and has a Probe Idol on the battlefield; it casts the spell at the Idol, which resolves.
fn animated(spell: &str) -> Chars {
    let filler = table().find("Filler").unwrap();
    let mut deck = vec![table().find(spell).unwrap()];
    deck.extend(std::iter::repeat(filler).take(29));
    let mut game = Game::new(table(), &Setup { decks: vec![deck, vec![filler; 30]], seed: 1, shuffle: false, ..Setup::default() });
    let idol = game.state.push_obj(Obj::new(table().find("Probe Idol").unwrap(), PlayerId(0), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, idol);
    game.verify = true;
    let mut cast = false;
    for _ in 0..100 {
        game.run();
        if cast && game.state.stack.is_empty() {
            return game.chars(idol);
        }
        let Some(request) = game.pending().cloned() else { panic!("the game ended") };
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && !cast => {
                match options.iter().find(|o| o.kind == ActionKind::CastSpell) {
                    Some(option) => {
                        cast = true;
                        let mut action = Action::of(option);
                        action.targets = vec![Target::Object(idol)];
                        Answer::Act(action)
                    }
                    None => Answer::Pass,
                }
            }
            RequestKind::Priority { .. } => Answer::Pass,
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("{spell} did not resolve");
}

/// CR 205.1b: "a 2/2 Construct artifact creature" replaces the Elf creature type with Construct; it is still Kindred and an
/// artifact, and still an Equipment (not a creature type).
#[test]
fn an_animation_naming_creature_types_replaces_the_ones_it_had() {
    let c = animated("Probe Replace");
    assert!(c.is(CardType::Creature) && c.is(CardType::Artifact) && c.is(CardType::Kindred));
    assert!(c.has_subtype(Subtype::Construct));
    assert!(!c.has_subtype(Subtype::Elf), "the Elf creature type is replaced");
    assert!(c.has_subtype(Subtype::Equipment), "a subtype that is not a creature type stays");
}

/// One that says it is still what it was adds Construct beside Elf.
#[test]
fn an_animation_that_keeps_what_it_was_adds_the_creature_type() {
    let c = animated("Probe Keep");
    assert!(c.has_subtype(Subtype::Construct) && c.has_subtype(Subtype::Elf));
}
