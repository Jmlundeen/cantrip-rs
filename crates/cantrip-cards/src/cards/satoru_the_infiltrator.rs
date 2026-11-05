use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever Satoru and/or one or more other nontoken creatures you control enter, if none of them were cast or no mana
/// was spent to cast them" (CR 603.2c, 603.4, 603.6a): each entering Satoru, or other nontoken creature under its
/// controller's control, is one of the batch unless it was a spell cast with mana spent on it (CR 601.2h; a spell cast
/// without paying its mana cost has no mana spent, and a creature put onto the battlefield was not cast). Satoru and others
/// that entered together are one batch, so the ability triggers once, and a member that was cast with mana is no
/// member. A cast creature spell enters alone, so none of a batch is ever left out beside one that was.
fn enters_uncast_or_unpaid(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, new, to, .. } = event else { return None };
    let enters = if *new == me.source {
        to.is(ZoneKind::Battlefield)
    } else {
        triggers::another_nontoken_you_control_enters::<{ triggers::CREATURE_TYPE }>(q, me, event).is_some()
    };
    if enters && q.history().mana_spent_to_cast(*old).next().is_none() { Fired::about(Target::Object(*new)) } else { None }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, enters_uncast_or_unpaid),
        |g, item| effects::draw(g, item, 1),
        "Whenever Satoru and/or one or more other nontoken creatures you control enter, if none of them were cast or no mana was spent to cast them, draw a card.",
    )
    .once_per_batch()],
    ..oracle::SATORU_THE_INFILTRATOR
};
