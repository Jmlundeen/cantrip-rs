use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{AN_OPPONENT_DRAWS_A_CARD, YOU_DRAW_A_CARD};
use cantrip_oracle as oracle;

/// "They lose 2 life": the opponent who drew, the trigger's subject.
fn lose_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(player) = item.subject {
        g.lose_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOU_DRAW_A_CARD, |g, item| effects::gain_life(g, item, 2), "Whenever you draw a card, you gain 2 life."),
        triggered(AN_OPPONENT_DRAWS_A_CARD, lose_two, "Whenever an opponent draws a card, they lose 2 life."),
    ],
    ..oracle::SHEOLDRED_THE_APOCALYPSE
};
