use cantrip_api::*;
use cantrip_mechanics::{effects, select, targets};
use cantrip_oracle as oracle;

/// "Return up to three target creature cards with total mana value 3 or less from your graveyard to the battlefield." It may be
/// cast with no target (CR 601.2c). The total is judged as the targets are chosen and again on resolution: if it has grown past 3
/// every target is illegal (CR 608.2b); otherwise the targets still legal enter (CR 110.2a) and each gets a +1/+1 counter.
pub static CARD: CardDef = CardDef {
    spell: spell(
        &[TargetSpec { min: 0, max: 3, ..targets::CREATURE_CARD_IN_YOUR_GRAVEYARD }
            .together(select::total_mana_value_at_most(3))
            .labelled("Creature cards from your graveyard to return to the battlefield")],
        effects::return_targets_to_battlefield_with_plus_one_counters,
    ),
    ..oracle::SCOUT_FOR_SURVIVORS
};
