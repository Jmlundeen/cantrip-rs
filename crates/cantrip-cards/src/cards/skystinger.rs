use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Whenever this creature blocks a creature with flying" (CR 509.3b, 702.22h): each attacker it blocks, the ones of a band it
/// blocks too included (`declared: false`), judged as a creature with flying as the block is declared.
fn this_blocks_a_creature_with_flying(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Blocks { blocker, attacker, .. } if *blocker == source && targets::has_flying_creature(q, *attacker) => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::BLOCKS, this_blocks_a_creature_with_flying),
        effects::this_gets::<5, 0>,
        "Whenever this creature blocks a creature with flying, this creature gets +5/+0 until end of turn.",
    )],
    ..oracle::SKYSTINGER
};
