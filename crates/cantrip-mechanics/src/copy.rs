//! Copy effects many cards print (CR 707).

use cantrip_api::*;

/// What a permanent that may enter as a copy asks (CR 707.2): yes, and what it copies is chosen next.
const COPY_AS_ENTERS: Question =
    Question::new("CopyAsEnters", "enter as a copy?", "You choose what it copies next. If you don't, it enters as itself", "Choose a copy", "Enter as itself");

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
    enter_as_copy_asking(g, entry, eligible, exception, &COPY_AS_ENTERS)
}

/// `enter_as_copy`, asked in the card's own words: Vesuva's "enter tapped as a copy", where what the copy does as it enters
/// is part of the choice. Whether it enters as a copy is `entry.copy` afterwards.
pub fn enter_as_copy_asking(
    g: &mut dyn Game,
    entry: &mut Entering,
    eligible: fn(&Chars) -> bool,
    exception: fn(&dyn Query, Copiable) -> Copiable,
    question: &'static Question,
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
    if g.ask(controller, RequestKind::May { source: entering, purpose: Purpose::Ask(question) })? != Answer::Yes {
        return Ok(());
    }
    let chosen = if candidates.len() == 1 {
        candidates[0]
    } else {
        let ask = RequestKind::ChooseTarget { label: "", slot_player: None, total: None,
            source: entering,
            purpose: Purpose::CopyWhich,
            candidates: candidates.iter().map(|&id| Target::Object(id)).collect(),
            optional: false,
            least: 1,
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

/// The intervening "if it isn't a token" of "When this creature dies, if it isn't a token, create a token that's a copy
/// of it" (CR 603.4, 111.7): judged on the creature as it last existed on the battlefield (CR 603.10a). Use as
/// `TriggeredDef::condition` with `look_back: true`. Sphinx of False Conclusions and Vaultborn Tyrant print it.
pub fn not_a_token(q: &dyn Query, asker: &Asker) -> bool {
    !q.obj(asker.source).token
}

/// "Create a token that's a copy of it" (CR 707.2, 111.1) for the ability's controller: the copiable values of the
/// ability's object as it last existed, with `exception` applied for the copy's exceptions (CR 707.9b), as
/// `enter_as_copy` takes them. Sphinx of False Conclusions copies as is; Vaultborn Tyrant's copy is an artifact in
/// addition to its other types.
pub fn create_token_copy_of_it(
    g: &mut dyn Game,
    item: &StackItem,
    exception: fn(&dyn Query, Copiable) -> Copiable,
) -> Result<(), Suspend> {
    let values = g.copiable_values(item.object);
    let values = exception(g, values);
    g.create_token_copies_of(values, item.controller, 1)?;
    Ok(())
}

/// "Create a token that's a copy of target [permanent]" (CR 707.2, 111.1): the copiable values of the first target, for
/// the ability's or spell's controller. Nothing happens if that target is no longer legal (CR 608.2b). Esika's Chariot,
/// Octomancer, Three Steps Ahead and Caretaker's Talent print it.
pub fn create_token_copy_of_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(target) = crate::targets::object(g, item, 0) {
        g.create_token_copies(target, item.controller, 1)?;
    }
    Ok(())
}
