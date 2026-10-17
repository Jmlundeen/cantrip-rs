use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{5}: This land becomes a copy of target creature you control until end of turn. The "legend rule" doesn't apply to
/// permanents you control this turn." The copy takes the creature's copiable values - what it copies, if it is a copy
/// (CR 707.2, 707.3, the rulings) - not its counters, tapped state or attachments, and the land doesn't enter, so no
/// "enters" ability applies. Both effects last until end of turn and end together, in the cleanup step (CR 514.2), when
/// the legend rule applies again at once (CR 704.5j, 514.3a). With the target gone the ability does nothing at all
/// (CR 608.2b: the core doesn't resolve it). The land becomes the copy only while it is still this permanent (the core).
fn copy_and_lift_the_legend_rule(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(creature);
    g.become_copy(item.object, item.object, values, Duration::EndOfTurn);
    g.add_effect(item.object, item.object, EffectKind::NoLegendRule(item.controller), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{5}"), copy_and_lift_the_legend_rule, "{5}: This land becomes a copy of target creature you control until end of turn. The \"legend rule\" doesn't apply to permanents you control this turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    ..oracle::HALL_OF_ECHOES
};
