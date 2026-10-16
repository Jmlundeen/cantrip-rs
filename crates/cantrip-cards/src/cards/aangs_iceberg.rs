use cantrip_api::*;
use cantrip_mechanics::{scry, triggers::THIS_ENTERS, targets, exile_target_until_leaves};
use cantrip_oracle as oracle;

/// "Exile up to one other target nonland permanent until this enchantment leaves the battlefield" (CR 610.3).
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

/// "Sacrifice this enchantment. If you do, scry 2." The scry hangs on the sacrifice happening (CR 608.2c, 701.21a): an
/// Iceberg already gone, or not controlled by the ability's controller, scries nothing.
fn sacrifice_and_scry(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = item.object;
    if !is_live(g, this) || g.controller(this) != item.controller {
        return Ok(());
    }
    g.sacrifice(&[this]);
    if is_live(g, this) {
        return Ok(());
    }
    scry(g, item.controller, this, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_until_it_leaves, "When this enchantment enters, exile up to one other target nonland permanent until this enchantment leaves the battlefield.")
            .targets(&[up_to_one(targets::other_nonland_permanent_filter)]),
        ability(cost("{3}"), sacrifice_and_scry, "Waterbend {3}: Sacrifice this enchantment. If you do, scry 2. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)")
            .waterbend(),
    ],
    ..oracle::AANGS_ICEBERG
};
