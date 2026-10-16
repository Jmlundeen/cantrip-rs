use cantrip_api::*;
use cantrip_oracle as oracle;

/// Put a +1/+1 counter on each creature you control.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::lands::enters_tapped_unless_you_control_basic_land),
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(cost("{3}{W}, {T}"), counters, "{3}{W}, {T}: Put a +1/+1 counter on each creature you control."),
    ],
    ..oracle::ABANDONED_AIR_TEMPLE
};
