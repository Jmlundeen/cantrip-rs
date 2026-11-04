use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{1}{R}{G}, {T}: Exile this creature and target creature without flying that's attacking you." (CR 608.2, 701.13a).
fn exile_this_and_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut all = vec![item.object];
    if let Some(creature) = targets::object(g, item, 0)
        && creature != item.object
    {
        all.push(creature);
    }
    g.exile_all(&all);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}{R}{G}").and(TAP),
            exile_this_and_target,
            "{1}{R}{G}, {T}: Exile this creature and target creature without flying that's attacking you.",
        )
        .targets(&[targets::CREATURE_WITHOUT_FLYING_ATTACKING_YOU]),
    ],
    ..oracle::GIANT_TRAP_DOOR_SPIDER
};
