use cantrip_api::*;
use cantrip_mechanics::manifest_dread;
use cantrip_oracle as oracle;

/// "At the beginning of each opponent's upkeep" (CR 603.2b): the upkeep of a turn that is not its controller's.
fn opponents_upkeep(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::Upkeep } if q.state().turn.active != q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn manifest_dread(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    manifest_dread::manifest_dread(g, item.controller, item.object)
}

pub static CARD: CardDef = CardDef {
    // As an additional cost to cast this spell, exile six cards from your graveyard.
    additional_cost: Some(Cost { exile_from_graveyard: 6, ..Cost::NONE }),
    abilities: &[triggered(when(EventMask::STEP_BEGAN, opponents_upkeep), manifest_dread, "At the beginning of each opponent's upkeep, manifest dread. (Look at the top two cards of your library. Put one onto the battlefield face down as a 2/2 creature and the other into your graveyard. Turn it face up any time for its mana cost if it's a creature card.)")],
    ..oracle::ABHORRENT_OCULUS
};
