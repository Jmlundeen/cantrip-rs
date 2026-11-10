use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{2}, {T}: Put two charge counters on target artifact." (CR 122.1, 608.2b): the target must still be a legal artifact as
/// the ability resolves, or nothing is put on.
fn two_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(artifact, CounterKind::Charge, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}").and(TAP), two_charge_counters, "{2}, {T}: Put two charge counters on target artifact.")
            .targets(&[targets::ARTIFACT]),
        ability(cost("{3}").and(TAP), effects::target_creature_cant_be_blocked_this_turn, "{3}, {T}: Target creature you control can't be blocked this turn.")
            .targets(&[targets::CREATURE_YOU_CONTROL]),
        ability(cost("{4}").and(TAP), effects::draw_a_card_then_may_put_a_land_tapped, "{4}, {T}: Draw a card. You may put a land card from your hand onto the battlefield tapped."),
    ],
    ..oracle::JEFFERIES_TUBE
};
