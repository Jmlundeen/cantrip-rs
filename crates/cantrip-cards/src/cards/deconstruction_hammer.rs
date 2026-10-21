use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    // "Sacrifice Deconstruction Hammer" in the ability a Hammer gives (CR 118.3, 201.5a): the Hammer that gave this
    // instance, not another Hammer on the same creature.
    delayed: &[ability(cost("{3}, {T}").and(Cost { sacrifice_granter: true, ..Cost::NONE }), destroy_it, "{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Equipped creature has \"{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.\""),
    ],
    ..oracle::DECONSTRUCTION_HAMMER
};
