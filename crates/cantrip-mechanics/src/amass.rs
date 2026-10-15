//! Amass [subtype] N (CR 701.47a): "Put N +1/+1 counters on an Army you control. If you don't control an Army, create a
//! 0/0 black [subtype] Army creature token first. The Army becomes a [subtype] in addition to its other types."
//! Orcish Bowmasters amasses Orcs 1.

use crate::choose;
use cantrip_api::*;

/// Amass `subtype` `n`: `token` is the 0/0 black `subtype` Army creature token created when `controller` has no Army.
pub fn amass(
    g: &mut dyn Game,
    controller: PlayerId,
    source: ObjectId,
    token: &'static CardDef,
    subtype: Subtype,
    n: i16,
) -> Result<(), Suspend> {
    let armies = |g: &dyn Game| -> Vec<ObjectId> {
        g.permanents_of(controller)
            .into_iter()
            .filter(|&id| {
                let c = g.chars(id);
                c.is(CardType::Creature) && c.has_subtype(Subtype::Army)
            })
            .collect()
    };
    let mut candidates = armies(g);
    if candidates.is_empty() {
        let token = g.card_id(token);
        g.create_tokens(token, controller, 1, false)?;
        candidates = armies(g);
    }
    let army = match candidates.as_slice() {
        [only] => *only,
        _ => match choose::card(g, controller, source, Purpose::Keep, candidates)? {
            Some(army) => army,
            None => return Ok(()),
        },
    };
    g.add_counters(army, CounterKind::PlusOne, n);
    if !g.chars(army).has_subtype(subtype) {
        g.add_effect(source, army, EffectKind::AddSubtype(subtype), Duration::Indefinite);
    }
    Ok(())
}
