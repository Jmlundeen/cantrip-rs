use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::effects;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever equipped creature attacks, surveil 1." (CR 701.25a): looked at and put into the graveyard or left on top by
/// the Equipment's controller, whether or not it is still attached as the trigger resolves (CR 603.2c).
fn surveil_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::EQUIPPED_CREATURE_ATTACKS, surveil_one, "Whenever equipped creature attacks, surveil 1."),
        equip(mana("{1}"), "Equip {1}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<1, 1>, "Equipped creature gets +1/+1.")],
    ..oracle::TRICORDER
};
