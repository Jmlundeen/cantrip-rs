use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(cantrip_mechanics::triggers::EQUIPPED_CREATURE_ATTACKS, destroy_it, "Whenever equipped creature attacks, destroy target permanent.")
            .targets(&[targets::PERMANENT]),
        equip(mana("{6}"), "Equip {6}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<6, 6>, "Equipped creature gets +6/+6.")],
    ..oracle::ARGENTUM_ARMOR
};
