use cantrip_api::*;
use cantrip_mechanics::{attach::attached_object, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Enchant creature card in a graveyard", and, once its enters ability has returned the creature, "enchant creature
/// put onto the battlefield with this Aura" (CR 303.4a, 704.5m): the second form is legal exactly for the creature
/// that entered with this Aura. The graveyard card is gone by then (a new object), so the two never overlap.
fn enchant_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_graveyard(q, t, &[CardType::Creature])
        || ((targets::CREATURE.filter)(q, asker, t)
            && matches!(t, Target::Object(id) if q.obj(id).entered_with == Some(asker.source)))
}

/// Return enchanted creature card to the battlefield under your control and attach this Aura to it.
fn return_enchanted(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.obj(item.object).attached_to else { return Ok(()) };
    g.put_onto_battlefield_with_aura(card, item.controller, item.object)?;
    Ok(())
}

/// That creature's controller sacrifices it.
fn sacrifice_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let entered: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.obj(id).entered_with == Some(item.object))
        .collect();
    g.sacrifice(&entered);
    Ok(())
}

fn minus_one_power(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v - 1);
}

pub static CARD: CardDef = CardDef {
    enchant: Some(target(enchant_filter)),
    abilities: &[
        triggered(THIS_ENTERS, return_enchanted, "When Animate Dead enters, if it's on the battlefield, it loses \"enchant creature card in a graveyard\" and gains \"enchant creature put onto the battlefield with Animate Dead.\" Return enchanted creature card to the battlefield under your control and attach Animate Dead to it.")
            .condition(cantrip_mechanics::triggers::this_is_on_the_battlefield),
        triggered(cantrip_mechanics::triggers::THIS_LEAVES, sacrifice_creature, "When Animate Dead leaves the battlefield, that creature's controller sacrifices it.")
            .look_back(),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, minus_one_power, "Enchanted creature gets -1/-0.")],
    ..oracle::ANIMATE_DEAD
};
