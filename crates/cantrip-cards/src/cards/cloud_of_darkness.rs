use cantrip_api::*;
use cantrip_mechanics::{pump, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls gets -X/-X until end of turn, where X is the number of permanent cards in your graveyard"
/// (CR 110.4a, 608.2h): X is counted once, as the ability resolves, in its controller's graveyard (CR 109.5: "your"), and is a fixed
/// amount from then on (CR 611.2a). With the target illegal the core does not resolve the ability (CR 608.2b).
fn shrink_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let x = targets::permanent_cards_in_graveyard(g, item.controller) as i32;
    pump(g, item.object, creature, -x, -x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        shrink_target,
        "Particle Beam \u{2014} When this creature enters, target creature an opponent controls gets -X/-X until end of turn, where X is the number of permanent cards in your graveyard.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::CLOUD_OF_DARKNESS
};
