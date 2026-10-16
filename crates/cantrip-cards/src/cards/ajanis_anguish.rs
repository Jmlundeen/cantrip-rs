use cantrip_api::*;
use cantrip_mechanics::statics::{have_trample, your_creatures};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "When this enchantment enters, it deals X damage to any target": the X it was cast with (CR 107.3m), read from the
/// enchantment even when it has left the battlefield by the time the ability resolves (last-known information, CR 608.2h). Put
/// onto the battlefield without being cast, X is 0 and no damage is dealt.
fn deal_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = g.obj(item.object).x;
    damage_target(g, item, 0, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, deal_x, "When this enchantment enters, it deals X damage to any target.").targets(&[targets::ANY])],
    statics: &[static_ability(Layer::Ability, your_creatures, have_trample, "Creatures you control have trample.")],
    ..oracle::AJANIS_ANGUISH
};
