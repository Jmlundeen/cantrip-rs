use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one" (CR 700.2): a +1/+1 counter on target creature you control, which then gains hexproof until end of turn
/// (CR 122.1, 702.11, 611.2a), or "Destroy target creature with power 4 or greater" (CR 701.8). The power is checked as the spell
/// is cast and again as it resolves (CR 608.2b), so a target that has shrunk below 4 is not destroyed.
static MODAL: ModalDef = modes(&[
    mode(
        effects::plus_one_counters_on_first_target_that_gains_until_end_of_turn::<1, { Keywords::of(&[Keyword::Hexproof]).0 }>,
        "Put a +1/+1 counter on target creature you control. It gains hexproof until end of turn.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL]),
    mode(effects::destroy_first_target, "Destroy target creature with power 4 or greater.").targets(&[targets::CREATURE_WITH_POWER_4_OR_GREATER]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SPECTACULAR_TACTICS };
