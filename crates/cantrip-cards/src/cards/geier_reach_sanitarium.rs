use cantrip_api::*;
use cantrip_mechanics::{each_discards_a_card, effects};
use cantrip_oracle as oracle;

/// "Each player draws a card, then discards a card" (CR 101.4, 121.1, 701.9): everyone draws in turn order from the active
/// player; then each in that order chooses a card of their hand (the drawn card included) and all are discarded together.
fn draw_then_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::each_player_draws_a_card(g, item)?;
    g.end_sentence();
    let players: Vec<PlayerId> = g.state().apnap().collect();
    each_discards_a_card(g, &players, item.object)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}, {T}"), draw_then_discard, "{2}, {T}: Each player draws a card, then discards a card."),
    ],
    ..oracle::GEIER_REACH_SANITARIUM
};
