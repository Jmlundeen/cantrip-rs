use cantrip_api::*;
use cantrip_mechanics::{effects, targets, your_upkeep};
use cantrip_oracle as oracle;

/// "{U}{U}, {T}: Target creature gains shroud until end of turn. Activate only during your upkeep." (CR 608.2, 611.2a).
fn target_creature_gains_shroud_activate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    effects::gain_until_end_of_turn(g, item.object, target, Keyword::Shroud);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{U}{U}").and(TAP),
            target_creature_gains_shroud_activate,
            "{U}{U}, {T}: Target creature gains shroud until end of turn. Activate only during your upkeep.",
        )
        .targets(&[targets::CREATURE])
        .condition(your_upkeep),
    ],
    ..oracle::SVYELUNITE_PRIEST
};
