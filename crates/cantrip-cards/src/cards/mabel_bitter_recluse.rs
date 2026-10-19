use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Another target creature or planeswalker" (CR 115.1): not Mabel herself.
fn another_creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Planeswalker)
    })
}

/// "Remove up to three counters from another target creature or planeswalker." (CR 122.1) The counters are chosen as the
/// ability resolves, among the kinds it has.
fn remove_up_to_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    g.remove_counters_up_to(item.controller, item.object, permanent, 3)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        remove_up_to_three,
        "When Mabel enters, remove up to three counters from another target creature or planeswalker.",
    )
    .targets(&[target(another_creature_or_planeswalker)])],
    ..oracle::MABEL_BITTER_RECLUSE
};
