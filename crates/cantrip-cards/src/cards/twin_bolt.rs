use cantrip_api::*;
use cantrip_mechanics::{damage_divided, targets};
use cantrip_oracle as oracle;

/// "Twin Bolt deals 2 damage divided as you choose among one or two targets" (CR 601.2d): the targets and each one's
/// share are fixed as it is cast, and a target that has since become illegal is left out with its share (CR 608.2b).
fn twin_bolt_deals_2_damage_divided(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY.between(1, 2)], twin_bolt_deals_2_damage_divided),
    divide: Some(Divide { total: 2, reads: None }),
    ..oracle::TWIN_BOLT
};
