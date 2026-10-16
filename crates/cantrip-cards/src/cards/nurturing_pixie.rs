use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Up to one target non-Faerie, nonland permanent you control": the Faerie word is read through the words of Nurturing
/// Pixie's text (CR 612).
fn non_faerie_nonland_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::nonland_permanent_you_control_filter(q, asker, t)
        && matches!(t, Target::Object(id) if !q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Faerie)))
}

/// "Return up to one target ... to its owner's hand. If a permanent was returned this way, put a +1/+1 counter on this
/// creature." A target that became illegal is not returned (CR 608.2b); Pixie gets the counter only if the permanent
/// really arrived in a hand, and only if Pixie is still on the battlefield.
fn bounce_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    if !g.return_to_hand(&[permanent]).is_empty() {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, bounce_and_grow, "When this creature enters, return up to one target non-Faerie, nonland permanent you control to its owner's hand. If a permanent was returned this way, put a +1/+1 counter on this creature.")
        .targets(&[up_to_one(non_faerie_nonland_permanent_you_control)])],
    ..oracle::NURTURING_PIXIE
};
