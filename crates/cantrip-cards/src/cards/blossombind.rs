use cantrip_api::*;
use cantrip_mechanics::{attach, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Enchanted creature ... can't have counters put on it" (CR 122.6, 101.2) as a `CardDef::counter_bans` entry: counters of any
/// kind, put by an effect, as a cost (blight) or by damage from a source with wither or infect. The counters already on it stay.
fn enchanted_creature_cant_have_counters(q: &dyn Query, asker: &Asker, object: ObjectId, _: PlayerId, _: CounterKind) -> bool {
    q.obj(asker.source).attached_to == Some(object)
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(THIS_ENTERS, |g, item| {
            attach::tap_attached(g, item);
            Ok(())
        }, "When this Aura enters, tap enchanted creature.")],
    // "Enchanted creature can't become untapped" (CR 701.26b, 101.2): no untap step, effect or cost untaps it.
    cant_become_untapped: &[attach::attached_to_it],
    counter_bans: &[enchanted_creature_cant_have_counters],
    ..oracle::BLOSSOMBIND
};
