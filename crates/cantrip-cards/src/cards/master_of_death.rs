use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::this_enters;
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
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, surveil 2.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::surveil::<2>,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, if this card is in your graveyard, you may pay 1 life. If you do, return it to your hand.",
            listens: EventMask::STEP_BEGAN,
            matches: cantrip_mechanics::triggers::your_upkeep,
            condition: Some(in_graveyard),
            effect: pay_to_return,
            anywhere: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::MASTER_OF_DEATH
};
