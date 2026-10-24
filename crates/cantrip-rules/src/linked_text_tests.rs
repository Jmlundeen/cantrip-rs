//! Which card a face's text belongs to, as linked abilities know it (CR 607.1, 607.1b; `CardTable::linked_card`): either face
//! of a transforming double-faced card is that card, a modal double-faced card's back face is a face of its own, and either
//! door of a Room is the Room. An exile made by one face's ability is found from the other face of a transforming card,
//! never across a modal card's faces. Over test cards.

use crate::{Game, Setup};
use cantrip_api::*;

/// A transforming card's back face has no mana cost of its own (CR 202.3b, 712.8e).
fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
    Ok(())
}

/// The back face, with an ability of its own ("... cast a spell from among the cards exiled with this permanent", Hauken's
/// Insight's).
static TRANSFORMED: CardDef = CardDef {
    name: "Linked Test Transformed",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(2),
    toughness: Some(2),
    has_cost: false,
    abilities: &[ability(TAP, nothing, "{T}: Back.")],
    ..CardDef::BLANK
};
/// The front face, with the ability the back's is linked to ("... exile a card from your hand face down", Jacob Hauken's).
static TRANSFORMING: CardDef = CardDef {
    name: "Linked Test Transforming",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    back: Some(&TRANSFORMED),
    abilities: &[ability(TAP, nothing, "{T}: Front.")],
    ..CardDef::BLANK
};

/// A modal double-faced card's back face with a mana cost of its own (CR 712.3: a face that may be cast).
static MODAL_BACK: CardDef = CardDef { name: "Linked Test Modal Back", types: TypeSet::of(&[CardType::Artifact]), has_cost: true, ..CardDef::BLANK };
static MODAL: CardDef = CardDef { name: "Linked Test Modal", types: TypeSet::of(&[CardType::Artifact]), has_cost: true, back: Some(&MODAL_BACK), ..CardDef::BLANK };

/// A modal double-faced card whose back face is a land played as such (CR 712.12).
static LAND_BACK: CardDef = CardDef { name: "Linked Test Land Back", types: TypeSet::of(&[CardType::Land]), has_cost: false, ..CardDef::BLANK };
static LAND_MODAL: CardDef =
    CardDef { name: "Linked Test Land Modal", types: TypeSet::of(&[CardType::Artifact]), has_cost: true, back: Some(&LAND_BACK), back_land: true, ..CardDef::BLANK };

static LEFT_DOOR: CardDef = CardDef { name: "Linked Test Left Door", types: TypeSet::of(&[CardType::Enchantment]), subtypes: SubtypeSet::of(&[Subtype::Room]), has_cost: true, ..CardDef::BLANK };
static RIGHT_DOOR: CardDef = CardDef { name: "Linked Test Right Door", types: TypeSet::of(&[CardType::Enchantment]), subtypes: SubtypeSet::of(&[Subtype::Room]), has_cost: true, ..CardDef::BLANK };
static ROOM: CardDef = CardDef {
    name: "Linked Test Room",
    types: TypeSet::of(&[CardType::Enchantment]),
    subtypes: SubtypeSet::of(&[Subtype::Room]),
    split: Some((&LEFT_DOOR, &RIGHT_DOOR)),
    ..CardDef::BLANK
};

static EXILED: CardDef = CardDef { name: "Linked Test Exiled", types: TypeSet::of(&[CardType::Creature]), power: Some(1), toughness: Some(1), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    Box::leak(Box::new(CardTable::new([&TRANSFORMING, &MODAL, &LAND_MODAL, &ROOM, &LEFT_DOOR, &RIGHT_DOOR, &EXILED])))
}

#[test]
fn each_face_belongs_to_the_card_its_text_is_printed_on() {
    let t = table();
    let id = |def: &'static CardDef| t.id_of(def);
    let back = |front: &'static CardDef| t.face(id(front), true);
    // A transforming double-faced card: both faces are one text (CR 607.1b).
    assert_ne!(back(&TRANSFORMING), id(&TRANSFORMING), "the transforming card has a back face");
    assert_eq!(t.linked_card(back(&TRANSFORMING)), id(&TRANSFORMING));
    assert_eq!(t.linked_card(id(&TRANSFORMING)), id(&TRANSFORMING));
    // A modal double-faced card's back face, whether it has a mana cost or is a land played as such, is a face of its own.
    assert_eq!(t.linked_card(back(&MODAL)), back(&MODAL));
    assert_ne!(back(&MODAL), id(&MODAL));
    assert_eq!(t.linked_card(back(&LAND_MODAL)), back(&LAND_MODAL));
    assert_ne!(back(&LAND_MODAL), id(&LAND_MODAL));
    // Each door of a Room is the Room (CR 607.1, 709.5).
    assert_eq!(t.linked_card(id(&LEFT_DOOR)), id(&ROOM));
    assert_eq!(t.linked_card(id(&RIGHT_DOOR)), id(&ROOM));
    assert_eq!(t.linked_card(id(&ROOM)), id(&ROOM));
}

/// `card` on the battlefield, transformed or not, with a card exiled by the ability of its text as it is now.
fn exiled_by(card: &'static CardDef) -> (Game, ObjectId, ObjectId) {
    let t = table();
    let mut game = Game::new(t, &Setup { decks: vec![Vec::new(); 2], ..Default::default() });
    let me = PlayerId(0);
    let source = game.state.push_obj(Obj::new(t.id_of(card), me, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, source);
    let exiled = game.state.push_obj(Obj::new(t.id_of(&EXILED), PlayerId(1), Zone::exile(PlayerId(1))));
    game.state.zone_push(Zone::exile(PlayerId(1)), exiled);
    let by = game.linked_to(source);
    game.state.update_obj(exiled, |o| o.exiled = Some(ExileLink::new(by, Zone::graveyard(PlayerId(1)), false)));
    (game, source, exiled)
}

#[test]
fn an_exile_by_one_face_is_found_from_the_other_face_of_a_transforming_card_only() {
    // Exiled by the front face's ability, found by the back face's once it transforms (CR 607.1b).
    let (mut game, source, exiled) = exiled_by(&TRANSFORMING);
    assert_eq!(game.exiled_with(game.linked_to(source)), vec![exiled]);
    game.state.update_obj(source, |o| o.transformed = true);
    assert_eq!(game.exiled_with(game.linked_to(source)), vec![exiled], "the back face's ability finds what the front face's exiled");
    // A modal double-faced card's faces are two texts: what one face's ability exiled is not the other's.
    let (mut game, source, _) = exiled_by(&MODAL);
    game.state.update_obj(source, |o| o.transformed = true);
    assert!(game.exiled_with(game.linked_to(source)).is_empty(), "the other face of a modal card finds nothing");
}

/// A transforming card's two faces are one text (CR 607.1b), but an ability of its front face an effect gives it while its
/// back face is up was acquired by that effect, and is linked to nothing it prints (CR 607.5): Agatha's Soul Cauldron lending
/// Jacob Hauken's ability to an animated Hauken's Insight. The back face's own ability is its own.
#[test]
fn a_front_face_ability_given_to_the_back_face_is_acquired_by_the_effect() {
    let (mut game, source, _) = exiled_by(&TRANSFORMING);
    let t = game.cards;
    game.state.update_obj(source, |o| o.transformed = true);
    // Given by another object's effect, as the Cauldron's.
    let lender = game.state.push_obj(Obj::new(t.id_of(&EXILED), PlayerId(0), Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, lender);
    let timestamp = game.state.timestamp();
    game.state.effect_push(ContinuousEffect {
        source: lender,
        controller: PlayerId(0),
        timestamp,
        affected: source,
        kind: EffectKind::AddAbility(AbilityRef { card: t.id_of(&TRANSFORMING), index: 0 }),
        duration: Duration::EndOfTurn,
    });
    let chars = crate::layers::chars(t, &game.state, &game.log, source);
    let slot = |card: &'static CardDef| chars.abilities.iter().position(|a| a.is_some_and(|a| a.card == t.id_of(card))).expect("the ability");
    let given = crate::layers::acquired(t, &game.state, &game.log, source, &chars, slot(&TRANSFORMING));
    assert_eq!(given, Acquired { by: AcquiredBy::Grant, stamp: timestamp, part: 0 }, "the given front-face ability is the effect's");
    let printed = crate::layers::acquired(t, &game.state, &game.log, source, &chars, slot(&TRANSFORMED));
    assert_eq!(printed, Acquired::OWN, "the back face's printed ability is its own");
}

/// "+1/+0 while a color was chosen for it", read by the ability that asks (CR 607.2d): a static ability of `CHOOSER`'s text,
/// which the card prints and also lists to give (`delayed`).
fn plus_one_for_a_chosen_color(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    if q.state().chosen_color(q.linked_to(source)).is_some() {
        chars.power = chars.power.map(|p| p + 1);
    }
}

static CHOOSER: CardDef = CardDef {
    name: "Linked Test Chooser",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    statics: &[characteristic_defining(Layer::PtModify, plus_one_for_a_chosen_color, "This creature gets +1/+0 while a color was chosen.")],
    delayed: &[granted_static(Layer::PtModify, plus_one_for_a_chosen_color, "This creature gets +1/+0 while a color was chosen.")],
    ..CardDef::BLANK
};

/// A static ability an effect gives a permanent is acquired by that effect (CR 607.5), even where the permanent prints the
/// same one: linked only to the abilities that effect gave it, it reads no choice the permanent's own text made (CR 607.2d).
/// The Chooser chose a color as it entered: its printed "+1/+0" reads the choice; the same ability another object's effect
/// gave it reads none, so it is a 2/1, not a 3/1.
#[test]
fn a_static_ability_given_by_an_effect_reads_no_choice_the_permanents_own_text_made() {
    let t = Box::leak(Box::new(CardTable::new([&CHOOSER, &EXILED])));
    let mut game = Game::new(t, &Setup { decks: vec![Vec::new(); 2], ..Default::default() });
    let me = PlayerId(0);
    let source = game.state.push_obj(Obj::new(t.id_of(&CHOOSER), me, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, source);
    let text = t.linked_card(t.id_of(&CHOOSER));
    game.state.update_obj(source, |o| {
        o.chosen_color = Some(Color::Blue);
        o.marks.chosen_by = Some(text);
    });
    assert_eq!(crate::layers::chars(t, &game.state, &game.log, source).power, Some(2), "its own ability reads its own choice");
    let lender = game.state.push_obj(Obj::new(t.id_of(&EXILED), me, Zone::BATTLEFIELD));
    game.state.zone_push(Zone::BATTLEFIELD, lender);
    let timestamp = game.state.timestamp();
    game.state.effect_push(ContinuousEffect {
        source: lender,
        controller: me,
        timestamp,
        affected: source,
        kind: EffectKind::AddAbility(AbilityRef::delayed(t.id_of(&CHOOSER), 0)),
        duration: Duration::EndOfTurn,
    });
    let chars = crate::layers::chars(t, &game.state, &game.log, source);
    assert!(chars.abilities.contains(&Some(AbilityRef::delayed(t.id_of(&CHOOSER), 0))), "it has the given ability");
    assert_eq!(chars.power, Some(2), "the given ability reads no choice: it made none");
}
