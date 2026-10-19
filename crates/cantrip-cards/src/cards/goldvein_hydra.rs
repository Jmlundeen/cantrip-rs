use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "When this creature dies, create a number of tapped Treasure tokens equal to its power." Its power as it last existed on the
/// battlefield (CR 608.2h, the ruling): the +1/+1 counters and any pump or other effect it had as it died. The tokens are made
/// together (CR 101.4) under the ability's controller, none for a power of 0 or less (CR 107.1b).
fn create_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = targets::power_or_last_known(g, item.object);
    if power > 0 {
        let treasure = g.card_id(&tokens::TREASURE);
        g.create_tokens(treasure, item.controller, power as u32, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(effects::enters_with_x_counters),
    abilities: &[triggered(THIS_DIES, create_treasures, "When this creature dies, create a number of tapped Treasure tokens equal to its power.").look_back()],
    ..oracle::GOLDVEIN_HYDRA
};
