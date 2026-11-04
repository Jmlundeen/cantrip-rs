use cantrip_api::*;
use cantrip_mechanics::{delayed, triggers};
use cantrip_oracle as oracle;

/// Whenever this creature blocks or becomes blocked by a non-Wall creature (CR 603.2).
fn blocks_or_becomes_blocked(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let other = triggers::other_creature_in_block_with_this(q, asker.source, event)?;
    let wall = q.words(asker.source).subtype(Subtype::Wall);
    if !q.chars(other).has_subtype(wall) {
        Fired::about(Target::Object(other))
    } else {
        None
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::BLOCKS, blocks_or_becomes_blocked),
        delayed::destroy_that_creature_at_end_of_combat,
        "Whenever this creature blocks or becomes blocked by a non-Wall creature, destroy that creature at end of combat.",
    )],
    delayed: &[delayed::DESTROY_THAT_CREATURE_AT_END_OF_COMBAT],
    ..oracle::COCKATRICE
};
