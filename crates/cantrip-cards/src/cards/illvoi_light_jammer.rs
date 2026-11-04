use cantrip_api::*;
use cantrip_mechanics::attach::{attach_to_target, attached_object, equip};
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Attach it to target creature you control. That creature gains hexproof until end of turn." If the target is no longer
/// legal the whole ability does nothing (CR 608.2b); otherwise the creature gains it whether or not the Equipment is still there to
/// be attached (CR 400.7), for the rest of the turn (CR 611.2a).
fn attach_and_gain_hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    attach_to_target(g, item)?;
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Hexproof);
    }
    Ok(())
}

/// Flash is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            attach_and_gain_hexproof,
            "When this Equipment enters, attach it to target creature you control. That creature gains hexproof until end of turn.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<1, 2>, "Equipped creature gets +1/+2.")],
    ..oracle::ILLVOI_LIGHT_JAMMER
};
