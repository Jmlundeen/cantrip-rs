//! Tokens created attacking through `Game::create_tokens_worded` (mobilize's path): tapped only when the effect says
//! "tapped and attacking" (CR 508.4), over test cards, since no printed card in content makes worded tokens that enter
//! attacking untapped.

use crate::{Cx, Game, Setup};
use cantrip_api::*;

static SOLDIER: CardDef = CardDef {
    name: "Attacking Test Soldier",
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

fn cx(game: &mut Game) -> Cx<'_> {
    Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None, producing: None }
}

/// Player 0's turn, in combat with attackers declared, making one token attacking player 1, `tapped` as given.
fn made(tapped: bool) -> (Game, ObjectId) {
    let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&SOLDIER])));
    let mut game = Game::new(table, &Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
    game.state.update_turn(|t| {
        t.active = PlayerId(0);
        t.set_step(Step::DeclareAttackers);
        t.attacked = true;
    });
    let card = table.id_of(&SOLDIER);
    let token = {
        let mut cx = cx(&mut game);
        cantrip_api::Game::create_tokens_worded(&mut cx, &[(card, PlayerId(0))], tapped, Some(Target::Player(PlayerId(1))), Words::NONE)
            .expect("nothing is asked")[0]
    };
    (game, token)
}

#[test]
fn a_worded_token_created_attacking_is_untapped_unless_the_effect_says_tapped() {
    let (game, token) = made(false);
    assert!(!game.state.obj(token).tapped);
    assert!(game.state.attackers.iter().any(|a| a.id == token));
    let (game, token) = made(true);
    assert!(game.state.obj(token).tapped);
    assert!(game.state.attackers.iter().any(|a| a.id == token));
}
