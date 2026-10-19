use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Awaken the Inferno deals 6 damage to target creature or planeswalker an opponent controls. Put a +1/+1
/// counter on up to one target creature you control." Resolve each legal target independently (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(target) = targets::object(g, item, 0) {
        g.damage(item.object, Target::Object(target), 6);
    }
    if let Some(creature) = targets::object(g, item, 1) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS, targets::CREATURE_YOU_CONTROL.optional()],
        effect,
    ),
    abilities: &[typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}")],
    ..oracle::AWAKEN_THE_INFERNO
};
