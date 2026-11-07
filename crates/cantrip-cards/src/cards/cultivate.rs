use cantrip_api::*;
use cantrip_mechanics::{choose, library, select};
use cantrip_oracle as oracle;

/// "Search your library for up to two basic land cards, reveal those cards, put one onto the battlefield tapped and the other into your hand, then shuffle." (CR 608.2, 400.7, 701.24a).
fn search_library_up_two_basic(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let shown_0 = library::searched_cards(g, item.controller);
    let can_0: Vec<ObjectId> = {
        let q: &dyn Query = g;
        shown_0
            .iter()
            .copied()
            .filter(|&id| q.chars(id).supertypes.basic && q.chars(id).is(CardType::Land))
            .collect()
    };
    let found_0 = library::search(
        g,
        item.controller,
        item.controller,
        item.object,
        SearchDestination::Hand,
        select::Selection::listed(select::Count::UpTo(2), &shown_0, can_0),
    )?;
    let objects_1 = found_0.clone();
    let mut owners_1: Vec<PlayerId> = objects_1
        .iter()
        .map(|o| g.state().obj(*o).owner)
        .collect();
    owners_1.dedup();
    for owner in owners_1 {
        let cards: Vec<ObjectId> = objects_1
            .iter()
            .copied()
            .filter(|o| g.state().obj(*o).owner == owner)
            .collect();
        g.reveal(owner, &cards);
    }
    let selected_2_candidates = found_0.clone();
    let selected_2 = if selected_2_candidates.len() <= 1 {
        selected_2_candidates
    } else {
        choose::select(
                g,
                item.controller,
                item.object,
                Purpose::PutOntoBattlefield,
                selected_2_candidates,
                select::Selection::of(select::Count::AsManyAsCan(1), &[]),
            )?
    };
    let objects_3 = selected_2
        .clone()
        .into_iter()
        .filter(|&o: &ObjectId| !g.state().obj(o).token)
        .collect::<Vec<ObjectId>>();
    let rows_3: Vec<(ObjectId, PlayerId, bool)> = objects_3
        .iter()
        .map(|&o| (o, item.controller, true))
        .collect();
    g.put_all_onto_battlefield_as(&rows_3)?;
    let objects_4 = found_0.clone();
    g.return_to_hand_revealed(&objects_4);
    g.shuffle(item.controller);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search_library_up_two_basic),
    ..oracle::CULTIVATE
};
