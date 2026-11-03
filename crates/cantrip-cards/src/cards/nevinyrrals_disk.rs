use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{1}, {T}: Destroy all artifacts, creatures, and enchantments." (CR 608.2, 701.8a).
fn destroy_all_artifacts_creatures_enchantments(
    g: &mut dyn Game,
    _: &StackItem,
) -> Result<(), Suspend> {
    let objects_1 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                q.chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Creature)
                    || q.chars(id).is(CardType::Enchantment)
            })
            .collect::<Vec<ObjectId>>()
    };
    g.destroy_all(&objects_1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}").and(TAP),
            destroy_all_artifacts_creatures_enchantments,
            "{1}, {T}: Destroy all artifacts, creatures, and enchantments.",
        ),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::NEVINYRRALS_DISK
};
