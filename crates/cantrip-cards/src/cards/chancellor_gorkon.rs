use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When Chancellor Gorkon dies, exile up to two target creatures." (CR 608.2, 701.13a).
fn exile_up_two_target_creatures(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let t_0 = announced(g, item, 0, true);
    let objects_0 = t_0.iter().filter_map(|t| t.object()).collect::<Vec<ObjectId>>();
    g.exile_all(&objects_0);
    Ok(())
}

/// "When Chancellor Gorkon dies, exile up to two target creatures." (CR 608.2).
fn announced(g: &dyn Game, item: &StackItem, spec: u8, legal: bool) -> Vec<Target> {
    (0..g.state().targets(item).len())
        .filter(|&i| {
            g.state().spec(item, i) == spec && (!legal || g.target_legal(item, i))
        })
        .map(|i| g.state().target(item, i))
        .collect()
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                triggers::THIS_DIES,
                exile_up_two_target_creatures,
                "When Chancellor Gorkon dies, exile up to two target creatures.",
            )
            .targets(&[targets::CREATURE.between(0, 2)]),
    ],
    ..oracle::CHANCELLOR_GORKON
};
