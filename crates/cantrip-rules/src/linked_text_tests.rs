//! Which card a face's text belongs to, as linked abilities know it (CR 607.1, 607.1b; `CardTable::linked_card`): either face
//! of a transforming double-faced card is that card, a modal double-faced card's back face is a face of its own, and either
//! door of a Room is the Room. An exile made by one face's ability is found from the other face of a transforming card,
//! never across a modal card's faces. Over test cards.

use crate::{Game, Setup};
use cantrip_api::*;

/// A transforming card's back face has no mana cost of its own (CR 202.3b, 712.8e).
static TRANSFORMED: CardDef =
    CardDef { name: "Linked Test Transformed", types: TypeSet::of(&[CardType::Creature]), power: Some(2), toughness: Some(2), has_cost: false, ..CardDef::BLANK };
static TRANSFORMING: CardDef = CardDef {
    name: "Linked Test Transforming",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    back: Some(&TRANSFORMED),
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
    game.state.update_obj(exiled, |o| o.exiled = Some(ExileLink { by: by.object, text: by.text, from: Zone::graveyard(PlayerId(1)), until_leaves: false, encoded: false }));
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
