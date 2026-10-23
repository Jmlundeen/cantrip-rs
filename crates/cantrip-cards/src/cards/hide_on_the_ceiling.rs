use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Exile X target artifacts and/or creatures. Return the exiled cards to the battlefield under their owners' control at the
/// beginning of the next end step." X is announced with the spell and counts the targets (CR 601.2b-c); those still legal as
/// it resolves are exiled together (CR 608.2b, 406.2, 101.4). The delayed triggered ability (CR 603.7) is about the cards
/// exiled; at the end step it returns those still in exile (`delayed::return_exiled_to_battlefield`).
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exile_all(&targets::legal_objects(g, item));
    delayed::at_next_end_step_about(g, item, 0, &exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE.counted()], exile),
    delayed: &[delayed_trigger(delayed::return_exiled_to_battlefield, "Return the exiled cards to the battlefield under their owners' control.")],
    ..oracle::HIDE_ON_THE_CEILING
};
