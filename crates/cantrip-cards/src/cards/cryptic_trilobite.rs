use cantrip_api::*;
use cantrip_mechanics::effects::enters_with_x_counters;
use cantrip_oracle as oracle;

/// "{1}, {T}: Put a +1/+1 counter on this creature." Nothing if it has left the battlefield (CR 400.7): `add_counters`
/// puts none on an object that is gone.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_counters),
    abilities: &[
        mana_ability(remove_counters(CounterKind::PlusOne, 1), add_n(Mana::C, 2).only_for(ManaRestriction::ABILITIES), "Remove a +1/+1 counter from this creature: Add {C}{C}. Spend this mana only to activate abilities."),
        ability(cost("{1}, {T}"), grow, "{1}, {T}: Put a +1/+1 counter on this creature."),
    ],
    ..oracle::CRYPTIC_TRILOBITE
};
