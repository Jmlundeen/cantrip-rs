use cantrip_api::*;
use cantrip_mechanics::{counters, effects, lands, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this enchantment unless you pay {B}." (CR 701.21a, 118.1).
fn sacrifice_enchantment_unless_pay_b(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{B}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_enchantment_unless_pay_b,
            "At the beginning of your upkeep, sacrifice this enchantment unless you pay {B}.",
        ),
        counters::damage_unless_counter_when_player_puts::<{ lands::sub(Subtype::Forest) }>(
            "Whenever a player puts a Forest onto the battlefield, this enchantment deals 3 damage to that player unless they put a -1/-1 counter on a creature they control.",
        ),
    ],
    ..oracle::TOURACHS_CHANT
};
