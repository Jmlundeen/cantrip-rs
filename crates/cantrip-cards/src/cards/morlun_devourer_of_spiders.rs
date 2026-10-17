use cantrip_api::*;
use cantrip_mechanics::effects::enters_with_x_counters;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "When Morlun enters, he deals X damage to target opponent" (CR 107.3m, 120.3): the X his spell was cast with, carried to the
/// ability (0 if he was put onto the battlefield without being cast). He deals it even if he has left the battlefield by
/// then, as he last was there (CR 608.2h), so his lifelink still gains life.
fn deal_x_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x;
    damage_target(g, item, 0, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_counters),
    abilities: &[triggered(THIS_ENTERS, deal_x_damage, "When Morlun enters, he deals X damage to target opponent.").targets(&[targets::OPPONENT])],
    ..oracle::MORLUN_DEVOURER_OF_SPIDERS
};
