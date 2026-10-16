use cantrip_api::*;
use cantrip_mechanics::{delayed, exile_target_until_leaves, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Artifact or creature": a permanent on the battlefield that is either.
fn artifact_or_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if is_live(q, id) && q.is_on_battlefield(id) && {
            let c = q.chars(id);
            c.is(CardType::Artifact) || c.is(CardType::Creature)
        })
}

/// "Exile up to one target artifact or creature until this enchantment leaves the battlefield" (CR 610.3): it comes
/// back as soon as this is gone, and is not exiled at all if this already is.
fn exile_until_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

/// Channel: "Exile target artifact or creature. Return it to the battlefield under its owner's control at the
/// beginning of the next end step." The delayed ability is made even if the target is gone by then (CR 603.7).
fn channel(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        let exiled = g.exile_with(&[permanent], item.object, false);
        if !exiled.is_empty() {
            delayed::at_next_end_step(g, item, 0, Target::None);
        }
    }
    Ok(())
}

/// Return what the channel exiled to the battlefield, each under its owner's control (CR 110.2a).
fn return_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId)> =
        g.exiled_with(item.object).into_iter().map(|card| (card, g.obj(card).owner)).collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_until_leaves, "When this enchantment enters, exile up to one target artifact or creature until this enchantment leaves the battlefield.")
            .targets(&[up_to_one(artifact_or_creature)]),
        ability(cost("{1}{W}").and(DISCARD_THIS), channel, "Channel \u{2014} {1}{W}, Discard this card: Exile target artifact or creature. Return it to the battlefield under its owner's control at the beginning of the next end step.")
            .targets(&[target(artifact_or_creature)])
            .from(ZoneKind::Hand),
    ],
    delayed: &[delayed_trigger(return_exiled, "Return it to the battlefield under its owner's control.")],
    ..oracle::TOUCH_THE_SPIRIT_REALM
};
