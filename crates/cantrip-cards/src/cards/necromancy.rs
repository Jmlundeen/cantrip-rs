use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Enchant creature put onto the battlefield with Necromancy" (CR 303.4a, 704.5m): legal exactly for the creature that entered
/// with this permanent as its Aura.
fn enchant_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).entered_with == Some(asker.source))
}

/// It becomes an Aura with "enchant creature put onto the battlefield with Necromancy": the Aura subtype (CR 613.1d) and the
/// enchant ability (CR 613.1f, `becomes_aura_enchant`), kept as long as it is this object. Put target creature card from a
/// graveyard onto the battlefield under your control and attach Necromancy to it, as part of the same move.
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        g.add_effect(item.object, item.object, EffectKind::AddSubtype(Subtype::Aura), Duration::Indefinite);
        g.add_effect(item.object, item.object, EffectKind::Enchant(item.ability.card), Duration::Indefinite);
    });
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield_with_aura(card, item.controller, item.object)?;
    }
    Ok(())
}

/// That creature's controller sacrifices it.
fn sacrifice_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let entered: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.obj(id).entered_with == Some(item.object)).collect();
    g.sacrifice(&entered);
    Ok(())
}

/// "If you cast it any time a sorcery couldn't have been cast, the controller of the permanent it becomes sacrifices it at the
/// beginning of the next cleanup step."
fn cast_untimely(q: &dyn Query, id: ObjectId) -> bool {
    q.obj(id).marks.cast_untimely
}

pub static CARD: CardDef = CardDef {
    cast_as_though_flash: true,
    sacrifice_at_cleanup: Some(cast_untimely),
    becomes_aura_enchant: Some(target(enchant_filter)),
    abilities: &[
        triggered(triggers::THIS_ENTERS, reanimate, "When this enchantment enters, if it's on the battlefield, it becomes an Aura with \"enchant creature put onto the battlefield with Necromancy.\" Put target creature card from a graveyard onto the battlefield under your control and attach this enchantment to it.")
            .targets(&[targets::CREATURE_CARD_IN_A_GRAVEYARD])
            .condition(triggers::this_is_on_the_battlefield),
        triggered(triggers::THIS_LEAVES, sacrifice_creature, "When this enchantment leaves the battlefield, that creature's controller sacrifices it."),
    ],
    ..oracle::NECROMANCY
};
