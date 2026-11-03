use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Whenever this creature blocks or becomes blocked" (CR 509.3a, 509.3c): it blocks (its first declared block), or it becomes
/// blocked (CR 509.1h, the transition once, however many creatures block it, or an effect makes it blocked).
fn when_this_blocks_or_becomes_blocked(_: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    match *event {
        Event::Blocks { blocker, declared: true, .. } if blocker == asker.source => Fired::PLAIN,
        Event::BecameBlocked { attacker } if attacker == asker.source => Fired::PLAIN,
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask(EventMask::BLOCKS.0 | EventMask::BECAME_BLOCKED.0), when_this_blocks_or_becomes_blocked),
        effects::pump_self::<2, 2>,
        "Whenever this creature blocks or becomes blocked, it gets +2/+2 until end of turn.",
    )],
    ..oracle::CHUB_TOAD
};
