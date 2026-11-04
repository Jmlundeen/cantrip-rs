use cantrip_api::*;
use cantrip_mechanics::{delayed, targets};
use cantrip_oracle as oracle;

/// "Its controller sacrifices it at the beginning of the next end step." (CR 603.7c, 701.21a).
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        let controller = g.controller(creature);
        g.sacrifice(controller, &[creature]);
    }
    Ok(())
}

/// "{3}, {T}: Target creature you control gets +3/+3 until end of turn. Its controller sacrifices it at the beginning of the next end step." (CR 608.2, 611.2a).
fn target_creature_control_gets_3(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    cantrip_mechanics::pump(g, item.object, creature, 3, 3);
    g.end_sentence();
    delayed::at_next_end_step(g, item, 0, Target::Object(creature));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{3}").and(TAP),
                target_creature_control_gets_3,
                "{3}, {T}: Target creature you control gets +3/+3 until end of turn. Its controller sacrifices it at the beginning of the next end step.",
            )
            .targets(&[targets::CREATURE_YOU_CONTROL]),
    ],
    delayed: &[
        delayed_trigger(
            sacrifice_it,
            "Its controller sacrifices it at the beginning of the next end step.",
        ),
    ],
    ..oracle::CELESTIAL_SWORD
};
