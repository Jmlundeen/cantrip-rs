use cantrip_api::*;
use cantrip_mechanics::{pump, typecycling};
use cantrip_oracle as oracle;

/// "{2}: This creature gets +1/-1 until end of turn."
fn shift_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, -1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{2}"), shift_power, "{2}: This creature gets +1/-1 until end of turn."),
        typecycling::basic_landcycling(mana("{2}"), "Basic landcycling {2}"),
    ],
    ..oracle::UNDULATING_WITNESS
};
