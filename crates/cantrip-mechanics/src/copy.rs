//! Copy effects many cards print (CR 707).

use cantrip_api::*;

/// "You may have this [permanent] enter as a copy of any [permanent that passes `eligible`] on the battlefield, except
/// [`exception`]" (CR 614.1c, 707.2, 707.9): a choice, not a target (CR 608.2d), so hexproof and protection narrow nothing
/// and an opponent's permanent is as available as your own. With nothing to copy nothing is asked, and with one candidate which to copy is not. The exception becomes part of the copy's own copiable values (CR 707.9b),
/// so a later copy of it has it too. Use it as `CardDef::enters`. Phantasmal Image (a creature; an Illusion with a
/// sacrifice ability) and Phyrexian Metamorph (an artifact or a creature; an artifact too) print it.
pub fn enter_as_copy(
    g: &mut dyn Game,
    entry: &mut Entering,
    eligible: fn(&Chars) -> bool,
    exception: fn(&dyn Query, Copiable) -> Copiable,
) -> Result<(), Suspend> {
    let entering = entry.object;
    let candidates: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| id != entering && eligible(&g.chars(id)))
        .collect();
    if candidates.is_empty() {
        return Ok(());
    }
    let controller = g.obj(entering).controller;
    if g.ask(controller, RequestKind::May { source: entering, purpose: Purpose::CopyAsEnters })? != Answer::Yes {
        return Ok(());
    }
    let chosen = if candidates.len() == 1 {
        candidates[0]
    } else {
        let ask = RequestKind::ChooseTarget {
            source: entering,
            purpose: Purpose::CopyWhich,
            candidates: candidates.iter().map(|&id| Target::Object(id)).collect(),
            optional: false,
            count: 1,
            any_number: false,
        };
        match g.ask(controller, ask)? {
            Answer::Targets(t) => t.first().and_then(|t| t.object()).unwrap_or(candidates[0]),
            _ => candidates[0],
        }
    };
    // The copied object's copiable values, its own copy effect's exceptions included (CR 707.2, 707.9b), and this copy's
    // exception on top.
    let values = g.copiable_values(chosen);
    entry.copy = Some(exception(g, values));
    Ok(())
}
