use cantrip_api::*;
use cantrip_mechanics::library::random_of;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Exile a permanent card from your graveyard at random, then create a tapped token that's a copy of that card. If the exiled card
/// is a land card, repeat this process" (CR 701.13, 707.2, 111.1, 700.7): the card is chosen at random among the permanent cards in
/// the controller's graveyard (CR 110.4a), and the token copies the card as it printed (CR 707.2: its copiable values in exile, with
/// {X} 0). A land card repeats the process on what is left; with no permanent card in the graveyard it ends.
fn exile_at_random_and_copy_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    loop {
        let cards: Vec<ObjectId> = g
            .state()
            .zone(Zone::graveyard(item.controller))
            .iter()
            .copied()
            .filter(|&id| targets::is_permanent_card(&g.chars(id)))
            .collect();
        let Some(card) = random_of(g, &cards) else { return Ok(()) };
        let exiled = g.exile(card);
        let land = g.chars(exiled).is(CardType::Land);
        let values = g.copiable_values(exiled);
        g.create_token_copies_tapped(values, item.controller, 1, true)?;
        g.end_sentence();
        if !land {
            return Ok(());
        }
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        exile_at_random_and_copy_tapped,
        "Whenever Sin enters or attacks, exile a permanent card from your graveyard at random, then create a tapped token that's a copy of that card. If the exiled card is a land card, repeat this process.",
    )],
    ..oracle::SIN_SPIRAS_PUNISHMENT
};
