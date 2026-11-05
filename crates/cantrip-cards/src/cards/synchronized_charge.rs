use cantrip_api::*;
use cantrip_mechanics::{counters_divided, effects, harmonize, targets};
use cantrip_oracle as oracle;

/// "Distribute two +1/+1 counters among one or two target creatures you control. Creatures you control with counters on them
/// gain vigilance and trample until end of turn." The shares were fixed as the spell went on the stack, each target getting
/// at least one (CR 601.2d). The second sentence is a separate instruction (CR 608.2c): the creatures with counters of any
/// kind are the ones that have them as it is done, the new counters included, and no creature that enters later gains
/// anything (CR 611.2c).
fn charge(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counters_divided(g, item, CounterKind::PlusOne);
    g.end_sentence();
    effects::creatures_you_control_where_gain(g, item, &[Keyword::Vigilance, Keyword::Trample], |g, id| g.state().obj(id).has_counters());
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL.between(1, 2)], charge),
    divide: Some(Divide { total: 2, reads: None }),
    ways: &[harmonize::way(mana("{4}{G}"), "Harmonize {4}{G}")],
    ..oracle::SYNCHRONIZED_CHARGE
};
