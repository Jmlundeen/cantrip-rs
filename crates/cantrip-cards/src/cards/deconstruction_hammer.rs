use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

fn plus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 1);
}

/// The equipped creature has the Equipment's delayed ability 0 (CR 613.1f): an activated ability the creature controls
/// and that is gone the moment the Equipment stops being attached to it.
fn grant_destroy(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

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
    abilities: &[equip(mana(1, 0, 0, 0, 0, 0), "Equip {1}")],
    delayed: &[ability(cost("{3}, {T}").and(sacrifice(1, granting_hammer)), destroy_it, "{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT])],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +1/+1.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_one,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has \"{3}, {T}, Sacrifice Deconstruction Hammer: Destroy target artifact or enchantment.\"",
            layer: Layer::Ability,
            affects: attached_object,
            apply: grant_destroy,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::DECONSTRUCTION_HAMMER
};
