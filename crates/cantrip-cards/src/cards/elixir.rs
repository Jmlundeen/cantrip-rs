use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Shuffle all nonland cards from your graveyard into your library. You gain life equal to the number of cards shuffled into your
/// library this way." (CR 701.24a, 400.7): the nonland cards of the controller's own graveyard go to the library at once, and the
/// library is shuffled even if none did. The life gained is the number that arrived there.
fn shuffle_nonland_cards_in_and_gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let cards: Vec<ObjectId> = g.state().zone(Zone::graveyard(player)).iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let shuffled = g.put_into_library(&cards, LibraryPlace::Top);
    g.shuffle(player);
    g.gain_life(player, shuffled.len() as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[ability(
        cost("{5}, {T}").and(EXILE_THIS),
        shuffle_nonland_cards_in_and_gain_life,
        "{5}, {T}, Exile this artifact: Shuffle all nonland cards from your graveyard into your library. You gain life equal to the number of cards shuffled into your library this way.",
    )],
    ..oracle::ELIXIR
};
