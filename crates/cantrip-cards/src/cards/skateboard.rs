use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::statics::have_haste;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Tap target permanent" (CR 701.26a); a target that became illegal is no longer tapped (CR 608.2b).
fn tap_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        g.tap(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, tap_target, "When this Equipment enters, tap target permanent.")
            .targets(&[targets::PERMANENT]),
        equip(mana("{1}"), "Equip {1}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 0>, "Equipped creature gets +1/+0."),
        static_ability(Layer::Ability, attached_object, have_haste, "Equipped creature has haste."),
    ],
    ..oracle::SKATEBOARD
};
