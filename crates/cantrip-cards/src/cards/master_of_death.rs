use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("ReturnToHand", "pay {cost} to return {subject} to your hand?", "It is in your graveyard", "Pay {cost}", "Leave it");

/// "At the beginning of your upkeep, if this card is in your graveyard."
fn in_graveyard(q: &dyn Query, asker: &Asker) -> bool {
    let obj = q.obj(asker.source);
    obj.became.is_none() && obj.zone == Zone::graveyard(asker.controller)
}

/// You may pay 1 life. If you do, return it to your hand. Only a player with the life is asked (CR 119.4).
fn pay_to_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if g.may_pay(item.controller, card, card, life(1), &ASK)? {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::surveil(g, item, 2), "When this creature enters, surveil 2."),
        triggered(cantrip_mechanics::triggers::YOUR_UPKEEP, pay_to_return, "At the beginning of your upkeep, if this card is in your graveyard, you may pay 1 life. If you do, return it to your hand.")
            .condition(in_graveyard)
            .anywhere(),
    ],
    ..oracle::MASTER_OF_DEATH
};
