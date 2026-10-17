use cantrip_api::*;
use cantrip_mechanics::triggers::{ward_discard, YOUR_END_STEP};
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Sacrifice a permanent unless you gained life this turn" (CR 608.2c, 701.21a). The check is made as the ability
/// resolves, not as it triggers (the ruling: it always triggers), and counts life gained by its controller any time this turn
/// (CR 119.3); a player who has gained life is asked nothing. Otherwise they choose a permanent they control (this creature
/// among them) and sacrifice it.
fn sacrifice_unless_gained_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if g.history().life_gained(you) > 0 {
        return Ok(());
    }
    let permanents = g.permanents_of(you);
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Sacrifice, "permanent", permanents.clone(), permanents, 1, false)?;
    g.sacrifice(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_discard("Ward\u{2014}Discard a card. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player discards a card.)"),
        triggered(YOUR_END_STEP, sacrifice_unless_gained_life, "Infusion \u{2014} At the beginning of your end step, sacrifice a permanent unless you gained life this turn."),
    ],
    ..oracle::TRAGEDY_FEASTER
};
