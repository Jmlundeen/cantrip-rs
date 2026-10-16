use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, if this card is in your graveyard."
fn in_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    let obj = q.obj(asker.source);
    obj.became.is_none() && obj.zone == Zone::graveyard(asker.controller)
}

/// You may pay 1 life. If you do, return it to your hand.
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if g.state().player(item.controller).life < 1 {
        return Ok(());
    }
    if effects::ask_may(g, item.controller, card, Purpose::ReturnToHand)? {
        g.lose_life(item.controller, 1);
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::surveil::<2>, "When this creature enters, surveil 2."),
        triggered(cantrip_mechanics::triggers::YOUR_UPKEEP, pay_to_return, "At the beginning of your upkeep, if this card is in your graveyard, you may pay 1 life. If you do, return it to your hand.")
            .condition(in_graveyard)
            .anywhere(),
    ],
    ..oracle::MASTER_OF_DEATH
};
