use cantrip_api::*;
use cantrip_mechanics::hand;
use cantrip_oracle as oracle;

/// "Draw a card. You may put a land card from your hand onto the battlefield." (CR 121.1, 608.2c; the 2019-01-25 ruling): the
/// draw comes first and is mandatory; the entry is optional, from that player's own hand, and is not a land play, so it neither
/// needs an unused land drop nor waits for the player's own turn.
fn draw_and_may_put_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    g.end_sentence();
    hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], draw_and_may_put_land), ..oracle::GROWTH_SPIRAL };
