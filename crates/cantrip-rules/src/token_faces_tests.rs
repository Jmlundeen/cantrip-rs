//! A token's own faces (CR 707.8a), over test cards: a token made as a copy of a double-faced permanent is a double-faced
//! token, entering with the copied face up, each face under the copy's exceptions, and it transforms; a token made as a copy
//! of one face alone is single-faced and does not. The scenarios show the same with Three Steps Ahead and Ardyn, the Usurper.

use crate::ops;
use crate::{Cx, Game, Setup};
use cantrip_api::*;

const YOU: PlayerId = PlayerId(0);

static BEAST: CardDef = CardDef {
    name: "Token Faces Test Beast",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(4),
    toughness: Some(4),
    ..CardDef::BLANK
};
static VILLAGER: CardDef = CardDef {
    name: "Token Faces Test Villager",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    back: Some(&BEAST),
    ..CardDef::BLANK
};

fn game() -> (Game, &'static CardTable) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&VILLAGER])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    game.state.update_turn(|t| {
        t.active = YOU;
        t.priority = YOU;
        t.set_step(Step::PrecombatMain);
    });
    (game, table)
}

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

/// The copiable values of the Villager transformed - its back face up, a double-faced permanent's - with "except its name
/// is Copycat": the token is the Villager's card back face up, the Beast under the exception, and it transforms to the
/// Villager, still named Copycat. Its definition is the face up, not the front face (`CardTable::current`).
#[test]
fn a_token_copy_of_a_transformed_double_faced_permanent_has_both_faces_under_its_exceptions() {
    let (mut game, table) = game();
    let values = Copiable::of(table.id_of(&BEAST)).with_faces(true).named("Copycat");
    let mut cx = cx(&mut game);
    let token = ops::create_token_copies(&mut cx, values, YOU, 1).expect("nothing is asked")[0];
    let obj = *cx.s().obj(token);
    assert_eq!(obj.card, table.id_of(&VILLAGER));
    assert!(obj.transformed);
    assert_eq!(cx.cards().current(&obj), table.id_of(&BEAST));
    assert_eq!(cx.chars(token).name, "Copycat");
    assert_eq!(cx.chars(token).power, Some(4));
    assert!(ops::turn_over(&mut cx, token).expect("nothing is asked"));
    let obj = *cx.s().obj(token);
    assert_eq!(cx.cards().current(&obj), table.id_of(&VILLAGER));
    assert_eq!(cx.chars(token).name, "Copycat");
    assert_eq!(cx.chars(token).power, Some(1));
}

/// The copiable values of the Villager's front face alone (a Clone's that copied it): the token has that one face, and
/// does not transform (CR 701.27c).
#[test]
fn a_token_copy_of_one_face_is_single_faced() {
    let (mut game, table) = game();
    let values = Copiable::of(table.id_of(&VILLAGER));
    let mut cx = cx(&mut game);
    let token = ops::create_token_copies(&mut cx, values, YOU, 1).expect("nothing is asked")[0];
    assert!(!ops::turn_over(&mut cx, token).expect("nothing is asked"));
    assert!(!cx.s().obj(token).transformed);
    assert_eq!(cx.chars(token).power, Some(1));
}
