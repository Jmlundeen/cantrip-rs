use cantrip_api::*;
use cantrip_mechanics::{delayed, exile_target_until_leaves, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

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
        let exiled = g.exile_with(&[permanent], g.linked(item), false);
        if !exiled.is_empty() {
            delayed::at_next_end_step(g, item, 0, Target::None);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_until_leaves, "When this enchantment enters, exile up to one target artifact or creature until this enchantment leaves the battlefield.")
            .targets(&[up_to_one(targets::artifact_or_creature_filter)]),
        ability(cost("{1}{W}").and(DISCARD_THIS), channel, "Channel \u{2014} {1}{W}, Discard this card: Exile target artifact or creature. Return it to the battlefield under its owner's control at the beginning of the next end step.")
            .targets(&[targets::ARTIFACT_OR_CREATURE])
            .from(ZoneKind::Hand),
    ],
    delayed: &[delayed_trigger(cantrip_mechanics::effects::return_exiled_to_battlefield, "Return it to the battlefield under its owner's control.")],
    ..oracle::TOUCH_THE_SPIRIT_REALM
};
