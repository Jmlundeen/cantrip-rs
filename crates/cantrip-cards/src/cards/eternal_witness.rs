use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Target card from your graveyard": any card in the graveyard of the ability's controller.
fn card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    cantrip_mechanics::targets::card_in_your_graveyard(q, asker, t, &[])
}

/// You may return the target card to your hand. Asked as the ability resolves, and only if the target is still legal.
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    if cantrip_mechanics::effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, return_it, "When this creature enters, you may return target card from your graveyard to your hand.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: card_in_your_graveyard, label: "" }])],
    ..oracle::ETERNAL_WITNESS
};
