use cantrip_api::*;
use cantrip_mechanics::{effects::{self, gain_indestructible_until_end_of_turn}, targets};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature you control. It gains indestructible until end of turn." Both happen only to a
/// target still legal as the spell resolves (CR 608.2b).
fn counter_and_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
        gain_indestructible_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
    mode(counter_and_indestructible, "Put a +1/+1 counter on target creature you control. It gains indestructible until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::ORIGIN_OF_METALBENDING };
