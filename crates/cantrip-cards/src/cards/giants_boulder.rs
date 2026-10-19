use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{destroy_target, scry, targets};
use cantrip_oracle as oracle;

/// "When this artifact enters, scry 2." (CR 701.22.)
fn scry_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    scry(g, item.controller, item.object, 2)
}

/// "Destroy target permanent." (CR 701.8, 608.2b.) The artifact was sacrificed as a cost, so it is never its own target.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        lands::ANY_COLOR_FOR_ONE,
        triggered(THIS_ENTERS, scry_two, "When this artifact enters, scry 2."),
        ability(cost("{7}, {T}").and(SACRIFICE_THIS), destroy, "{7}, {T}, Sacrifice this artifact: Destroy target permanent.")
            .targets(&[targets::PERMANENT]),
    ],
    ..oracle::GIANTS_BOULDER
};
