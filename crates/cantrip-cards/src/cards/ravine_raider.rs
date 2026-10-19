use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

/// "This creature gets +1/+1 until end of turn" (CR 611.2): each activation adds one (they stack), and only if it is still the
/// creature that activated (CR 400.7).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{B}"), grow, "{1}{B}: This creature gets +1/+1 until end of turn.")],
    ..oracle::RAVINE_RAIDER
};
