use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "At end of combat, destroy all creatures blocking or blocked by this creature." (CR 608.2, 701.8a).
fn destroy_all_creatures_blocking_blocked(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let q: &dyn Query = g;
    let mut creatures = q.creatures_blocking(item.object);
    creatures.extend(q.creatures_blocked_by(item.object));
    g.destroy_all(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::END_OF_COMBAT,
            destroy_all_creatures_blocking_blocked,
            "At end of combat, destroy all creatures blocking or blocked by this creature.",
        ),
    ],
    ..oracle::KJELDORAN_FROSTBEAST
};
