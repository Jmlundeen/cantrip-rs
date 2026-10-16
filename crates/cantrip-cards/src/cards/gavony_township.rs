use cantrip_api::*;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature you control.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}{W}{G}, {T}"), counters, "{2}{G}{W}, {T}: Put a +1/+1 counter on each creature you control."),
    ],
    ..oracle::GAVONY_TOWNSHIP
};
