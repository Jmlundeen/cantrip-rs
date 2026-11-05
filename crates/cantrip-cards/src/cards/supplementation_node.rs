use cantrip_api::*;
use cantrip_mechanics::statics::{grant_delayed_ability, your_artifact_creatures};
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

const TEXT: &str = "Whenever this creature deals combat damage to a player, put a +1/+1 counter on it.";

/// "Artifact creatures you control have [the trigger]" (CR 613.1f, 603.2): a layer 6 grant judged on what layers 1-5
/// made each permanent, so a creature that becomes an artifact creature has it, and an opponent's never does.
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    delayed: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, counter, TEXT)],
    statics: &[static_ability(
        Layer::Ability,
        your_artifact_creatures,
        grant_delayed_ability,
        "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, put a +1/+1 counter on it.\"",
    )],
    ..oracle::SUPPLEMENTATION_NODE
};
