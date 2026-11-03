use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Put a counter on the target, then make it an artifact in addition to its types (CR 122.1, 611.2a).
fn transmogrify(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.end_sentence();
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::AddTypes(TypeSet::of(&[CardType::Artifact])), Duration::Indefinite);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP.and(SACRIFICE_THIS),
        transmogrify,
        "{T}, Sacrifice this artifact: Put a +1/+1 counter on target nonartifact creature. That creature becomes an artifact in addition to its other types.",
    ).targets(&[targets::NONARTIFACT_CREATURE])],
    ..oracle::ASHNODS_TRANSMOGRANT
};
