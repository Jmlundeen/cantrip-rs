use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When Chancellor Gorkon dies, exile up to two target creatures." (CR 608.2b, 701.13a).
fn exile_up_two_target_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.exile_all(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_DIES, exile_up_two_target_creatures, "When Chancellor Gorkon dies, exile up to two target creatures.")
            .targets(&[targets::CREATURE.between(0, 2)]),
    ],
    ..oracle::CHANCELLOR_GORKON
};
