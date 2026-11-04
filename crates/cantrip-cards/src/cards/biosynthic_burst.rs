use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counters_on_first_target_that_gains_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on target creature you control. It gains reach, trample, and indestructible until end of turn. Untap it."
/// (CR 122.1, 611.2a, 701.26b): three instructions on the one target, in order; a target that is no longer legal gets none of them
/// (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    plus_one_counters_on_first_target_that_gains_until_end_of_turn::<1, { Keywords::of(&[Keyword::Reach, Keyword::Trample, Keyword::Indestructible]).0 }>(g, item)?;
    g.untap(creature);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], effect), ..oracle::BIOSYNTHIC_BURST };
