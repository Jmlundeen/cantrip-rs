use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, exile all other permanents you control": at once (CR 101.4), remembered as exiled with this
/// creature (CR 607.2a) for its leaves ability. "You" is the ability's controller as it resolves; "other" leaves this object
/// out, so if it has already left the battlefield every permanent its controller controls is exiled, and since its leaves
/// ability has already triggered and resolved (finding nothing), they stay exiled (the 2022-12-08 ruling).
fn exile_all_others(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let others: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| id != item.object).collect();
    if !others.is_empty() {
        g.exile_with(&others, item.object, false);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, exile_all_others, "When this creature enters, exile all other permanents you control."),
        triggered(triggers::THIS_LEAVES, effects::return_exiled_to_battlefield, "When this creature leaves the battlefield, return the exiled cards to the battlefield under their owners' control."),
    ],
    ..oracle::WORLDGORGER_DRAGON
};
