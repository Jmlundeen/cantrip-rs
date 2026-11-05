use cantrip_api::*;
use cantrip_mechanics::backup;
use cantrip_oracle as oracle;

/// "Backup 1 (CR 702.165a)": the +1/+1 counter, and for another creature this creature's printed ability below it,
/// "assigns combat damage equal to its toughness rather than its power" (CR 510.1a).
fn backup(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    backup::resolve(g, item, 1, &[Keyword::AssignsDamageByToughness], None)
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::AssignsDamageByToughness]),
    abilities: &[backup::trigger("When this creature enters, put a +1/+1 counter on target creature. If that's another creature, it gains the following ability until end of turn.", backup)],
    ..oracle::STREETWISE_NEGOTIATOR
};
