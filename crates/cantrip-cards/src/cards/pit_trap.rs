use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{2}, {T}, Sacrifice this artifact: Destroy target attacking creature without flying. It can't be regenerated." (CR 608.2, 701.8a).
fn destroy_target_attacking_creature_without(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target_no_regen(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{2}").and(TAP).and(SACRIFICE_THIS),
                destroy_target_attacking_creature_without,
                "{2}, {T}, Sacrifice this artifact: Destroy target attacking creature without flying. It can't be regenerated.",
            )
            .targets(&[targets::ATTACKING_CREATURE_WITHOUT_FLYING]),
    ],
    ..oracle::PIT_TRAP
};
