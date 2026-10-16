use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Sacrifice Deconstruction Hammer" in the ability a Hammer grants (CR 118.3, 201.5a): a Hammer that grants it is one
/// attached to the creature that has the ability (the ability's source), controlled by whoever pays.
fn granting_hammer(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.attached_to(id) == Some(asker.source) && q.card(q.chars(id).card).name == "Deconstruction Hammer"
}

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[equip(mana("{1}"), "Equip {1}")],
    delayed: &[ability(cost("{3}, {T}").and(sacrifice(1, granting_hammer)), destroy_it, "{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, cantrip_mechanics::statics::grant_delayed_ability, "Equipped creature has \"{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.\""),
    ],
    ..oracle::DECONSTRUCTION_HAMMER
};
