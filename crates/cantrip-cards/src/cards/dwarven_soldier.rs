use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::other_creature_in_block_with_this};
use cantrip_oracle as oracle;

/// "Whenever this creature blocks or becomes blocked by one or more Orcs" (CR 603.2).
fn when_creature_blocks_becomes_blocked_by(
    q: &dyn Query,
    asker: &Asker,
    event: &Event,
) -> Option<Fired> {
    let other = other_creature_in_block_with_this(q, asker.source, event)?;
    if !q.chars(other).has_subtype(q.words(asker.source).subtype(Subtype::Orc)) {
        return None;
    }
    Fired::about(Target::Object(other))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                when_asked(EventMask::BLOCKS, when_creature_blocks_becomes_blocked_by),
                effects::pump_self::<0, 2>,
                "Whenever this creature blocks or becomes blocked by one or more Orcs, this creature gets +0/+2 until end of turn.",
            )
            .once_per_batch(),
    ],
    ..oracle::DWARVEN_SOLDIER
};
