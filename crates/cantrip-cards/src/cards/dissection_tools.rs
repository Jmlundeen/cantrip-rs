use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip_cost};
use cantrip_mechanics::manifest_dread::manifest_dread_object;
use cantrip_mechanics::statics::{have_deathtouch_and_lifelink, plus_pt};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "When this Equipment enters, manifest dread, then attach this Equipment to that creature" (CR 701.62a, 701.3a). The
/// attach does nothing when the Equipment or the creature is gone or the creature is no longer a legal host (CR 701.3b,
/// 301.5c), and a library with no card manifests nothing.
fn manifest_and_attach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = manifest_dread_object(g, item.controller, item.object)? {
        g.attach(item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, manifest_and_attach, "When this Equipment enters, manifest dread, then attach this Equipment to that creature."),
        equip_cost(sacrifice(1, targets::filters::creature), "Equip\u{2014}Sacrifice a creature"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Equipped creature gets +2/+2."),
        static_ability(Layer::Ability, attached_object, have_deathtouch_and_lifelink, "Equipped creature has deathtouch and lifelink."),
    ],
    ..oracle::DISSECTION_TOOLS
};
