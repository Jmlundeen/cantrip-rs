use cantrip_api::*;
use cantrip_mechanics::effects::ask_may_return_this_card;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("ReturnToHand", "return it from your graveyard to your hand?", "", "Return it", "Leave it");

/// "At the beginning of your upkeep": its owner's.
fn your_upkeep(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Upkeep } if q.state().turn.active == q.obj(source).owner => Fired::PLAIN,
        _ => None,
    }
}

/// You may return this card from your graveyard to your hand: asked only while it is still the card that triggered.
fn return_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may_return_this_card(g, item, Purpose::Ask(&ASK))? {
        g.return_to_hand(&[item.object]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::STEP_BEGAN, your_upkeep), return_it, "At the beginning of your upkeep, you may return this card from your graveyard to your hand.")
        // It returns this card from the graveyard, so it functions only there (CR 113.6m).
        .functions_in(IN_A_GRAVEYARD)],
    ..oracle::SQUEE_GOBLIN_NABOB
};
