//! "Search your library for any number of [cards]" (CR 701.23) is asked as any number - `RequestKind::SearchLibrary`'s
//! `any_number`, as `ChooseCards` says it - with every card that qualifies as the count, never a count the clients must read
//! as "any number" by its size. A test card searches for any number of Islands, and takes all 300.

use std::sync::OnceLock;

use cantrip_api::Game as Rules;
use cantrip_engine::*;

fn any_islands(g: &mut dyn Rules, item: &StackItem) -> Result<(), Suspend> {
    let island = cantrip_api::Subtype::Island;
    let found = cantrip_mechanics::search_library(g, item.controller, item.controller, item.object, SearchDestination::Hand, cantrip_mechanics::ANY_NUMBER, true, |q, c| {
        q.chars(c).has_subtype(island)
    })?;
    g.return_to_hand(&found);
    Ok(())
}

static ANY_ISLANDS: CardDef = CardDef { name: "Probe Any Islands", types: TypeSet::of(&[CardType::Sorcery]), spell: spell(&[], any_islands), ..CardDef::BLANK };

fn table() -> &'static CardTable {
    static TABLE: OnceLock<CardTable> = OnceLock::new();
    TABLE.get_or_init(|| CardTable::new(cantrip_cards::all().into_iter().chain([&ANY_ISLANDS])))
}

#[test]
fn a_search_for_any_number_is_asked_as_any_number_and_takes_all_300() {
    let card = |name: &str| table().find(name).unwrap_or_else(|| panic!("no card {name}"));
    let mut deck = vec![card("Probe Any Islands")];
    deck.extend(vec![card("Island"); 306]);
    let mut game = Game::new(table(), &Setup { decks: vec![deck, vec![card("Island"); 30]], seed: 1, shuffle: false, opening_chooser: None, ..Default::default() });
    let mut asked = None;
    for _ in 0..200 {
        game.run();
        let request = game.pending().cloned().expect("a question");
        if asked.is_some() && game.state.stack.is_empty() {
            break;
        }
        let answer = match &request.kind {
            RequestKind::Priority { options } if request.player == PlayerId(0) && asked.is_none() && game.state.stack.is_empty() => options
                .iter()
                .find(|o| o.kind == ActionKind::CastSpell)
                .map_or(Answer::Pass, |o| Answer::Act(Action::of(o))),
            RequestKind::Priority { .. } => Answer::Pass,
            RequestKind::PayCosts { options, .. } => Answer::Act(Action::of(options.iter().find(|o| o.kind == ActionKind::ConfirmCasting).expect("nothing to pay"))),
            RequestKind::SearchLibrary { eligible, count, optional, any_number, .. } => {
                asked = Some((*any_number, *optional, *count, eligible.len()));
                Answer::Cards(eligible.clone())
            }
            other => panic!("unexpected question {other:?}"),
        };
        game.answer(request.id, request.player, answer).unwrap();
    }
    assert_eq!(asked, Some((true, true, 300, 300)));
    assert_eq!(game.state.zone(Zone::hand(PlayerId(0))).len(), 6 + 300);
}
