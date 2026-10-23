//! A token's name is the one the text that makes it gives it, or else its subtypes followed by "Token" (CR 111.4), so a
//! token named for its subtypes never has the name of a card (CR 201.2a): the Phyrexian Hydra Tokens Polukranos Reborn
//! makes are no Phyrexian Hydra, the card. No card in content is named for a token's subtypes, so the card here is a test
//! card that is.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;
use cantrip_mechanics::library::object_names;
use cantrip_mechanics::same_name;

/// A 3/3 creature card named Phyrexian Hydra, for no mana.
static HYDRA_CARD: CardDef = CardDef {
    name: "Phyrexian Hydra",
    oracle: true,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Phyrexian, Subtype::Hydra]),
    power: Some(3),
    toughness: Some(3),
    ..CardDef::BLANK
};

/// The token "a 3/3 green Phyrexian Hydra creature token" makes.
static HYDRA_TOKEN: CardDef = CardDef { types: TypeSet::of(&[CardType::Creature]), power: Some(3), toughness: Some(3), ..token!(Phyrexian Hydra) };

fn make_two(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let hydra = g.card_id(&HYDRA_TOKEN);
    g.create_tokens(hydra, item.controller, 2, false)?;
    Ok(())
}

/// A zero-cost sorcery: "Create two 3/3 green Phyrexian Hydra creature tokens."
static MAKER: CardDef = CardDef { name: "Hydra Maker", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], make_two), ..CardDef::BLANK };

static FILLER: CardDef = CardDef { name: "Filler", has_cost: false, types: TypeSet::of(&[CardType::Land]), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new([&HYDRA_CARD, &HYDRA_TOKEN, &MAKER, &FILLER]))
}

/// Seat 0 casts the Hydra card and then the maker, passing otherwise, until both have resolved.
fn played() -> Game {
    let deck: Vec<CardId> = ["Phyrexian Hydra", "Hydra Maker"].iter().map(|n| table().find(n).unwrap()).chain(std::iter::repeat(table().find("Filler").unwrap()).take(28)).collect();
    let filler = vec![table().find("Filler").unwrap(); 30];
    let mut game = Game::new(table(), &Setup { decks: vec![deck, filler], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    game.verify = true;
    for _ in 0..200 {
        game.run();
        if game.state.battlefield().len() == 3 && game.state.stack.is_empty() {
            return game;
        }
        let request = game.pending().cloned().expect("the game goes on");
        let answer = match &request.kind {
            RequestKind::Priority { options } => options
                .iter()
                .find(|o| o.kind == ActionKind::CastSpell && request.player == PlayerId(0))
                .map_or(Answer::Pass, |o| Answer::Act(Action::of(o))),
            RequestKind::May { .. } => Answer::Yes,
            RequestKind::DiscardToHandSize { count, hand } => Answer::Cards(hand.iter().copied().take(*count as usize).collect()),
            RequestKind::DeclareAttackers { .. } => Answer::Attack(Vec::new()),
            RequestKind::DeclareBlockers { .. } => Answer::Block(Vec::new()),
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    panic!("the Hydras never arrived");
}

#[test]
fn a_token_named_for_its_subtypes_does_not_share_a_cards_name() {
    let game = played();
    let named = |name: &str| game.state.battlefield().iter().copied().filter(|&id| game.name(id) == name).collect::<Vec<_>>();
    let cards = named("Phyrexian Hydra");
    let tokens = named("Phyrexian Hydra Token");
    assert_eq!((cards.len(), tokens.len()), (1, 2), "one Phyrexian Hydra and two Phyrexian Hydra Tokens");
    assert_eq!(object_names(&game, tokens[0]), vec!["Phyrexian Hydra Token"]);
    // Both are Phyrexian Hydras by type, and not by name (CR 201.2a, 111.4).
    assert!(game.chars(tokens[0]).has_subtype(Subtype::Hydra) && game.chars(cards[0]).has_subtype(Subtype::Hydra));
    assert!(!same_name(&game, cards[0], tokens[0]));
    assert!(!same_name(&game, tokens[1], cards[0]));
    assert!(same_name(&game, tokens[0], tokens[1]));
    // A card's name finds the card, never the token (CR 201.4).
    assert_eq!(table().find("Phyrexian Hydra"), Some(table().id_of(&HYDRA_CARD)));
}
