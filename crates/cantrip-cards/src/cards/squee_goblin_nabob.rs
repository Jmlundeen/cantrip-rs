use cantrip_api::*;
use cantrip_mechanics::effects::ask_may_return_this_card;
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep": its owner's, while it is in a graveyard.
fn upkeep_in_graveyard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let obj = q.obj(source);
    match event {
        Event::StepBegan { step: Step::Upkeep }
            if q.state().turn.active == obj.owner && obj.zone.is(ZoneKind::Graveyard) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// You may return this card from your graveyard to your hand: asked only while it is still the card that triggered.
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may_return_this_card(g, item, Purpose::ReturnToHand)? {
        g.return_to_hand(&[item.object]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::STEP_BEGAN, upkeep_in_graveyard), return_it, "At the beginning of your upkeep, you may return this card from your graveyard to your hand.")
        .anywhere()],
    ..oracle::SQUEE_GOBLIN_NABOB
};
