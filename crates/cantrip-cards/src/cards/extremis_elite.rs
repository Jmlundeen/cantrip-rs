use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, power_up, targets};
use cantrip_oracle as oracle;

/// "Put two +1/+1 counters on this creature. It deals 1 damage to any target." If the creature has left the
/// battlefield, it still deals the damage, as it last existed (CR 608.2h).
fn counters_and_ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::two_plus_one_counters_on_this(g, item)?;
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{4}{R}"),
        counters_and_ping,
        "Power-up \u{2014} {4}{R}: Put two +1/+1 counters on this creature. It deals 1 damage to any target. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )
    .targets(&[targets::ANY])],
    ..oracle::EXTREMIS_ELITE
};
