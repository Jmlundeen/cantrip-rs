use cantrip_api::*;
use cantrip_mechanics::{delayed, effects::gain_control_untap_haste, targets};
use cantrip_oracle as oracle;

/// "Gain control of target artifact or creature until end of turn. Untap that permanent. It gains haste until end of turn. If it's a
/// Spacecraft, put ten charge counters on it. If you do, remove ten charge counters from it at the beginning of the next end step"
/// (CR 613.1b, 611.2a, 702.10, 603.7): the counters make it a creature this turn (its station striations). "If you do" is whether
/// the counters were put, which a replacement (Solemnity) can stop; the removal is then not set up, so counters the Spacecraft
/// already had stay.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = gain_control_untap_haste(g, item) else { return Ok(()) };
    g.end_sentence();
    if !g.chars(permanent).has_subtype(g.words(item.object).subtype(Subtype::Spacecraft)) {
        return Ok(());
    }
    let before = g.state().obj(permanent).counters(CounterKind::Charge);
    g.add_counters(permanent, CounterKind::Charge, 10);
    if g.state().obj(permanent).counters(CounterKind::Charge) > before {
        delayed::at_next_end_step(g, item, 0, Target::Object(permanent));
    }
    Ok(())
}

/// The delayed ability: ten charge counters come off the Spacecraft, if it is still on the battlefield as the permanent it was
/// (CR 603.7c, 400.7).
fn remove_the_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(permanent) = item.subject {
        g.remove_counters(permanent, CounterKind::Charge, 10);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE], effect),
    delayed: &[delayed_trigger(remove_the_counters, "Remove ten charge counters from it.")],
    ..oracle::SYSTEMS_OVERRIDE
};
