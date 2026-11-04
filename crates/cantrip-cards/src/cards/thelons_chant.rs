use cantrip_api::*;
use cantrip_mechanics::{counters, effects, lands, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice this enchantment unless you pay {G}." (CR 701.21a, 118.1).
fn sacrifice_enchantment_unless_pay_g(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{G}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_enchantment_unless_pay_g,
            "At the beginning of your upkeep, sacrifice this enchantment unless you pay {G}.",
        ),
        counters::damage_unless_counter_when_player_puts::<{ lands::sub(Subtype::Swamp) }>(
            "Whenever a player puts a Swamp onto the battlefield, this enchantment deals 3 damage to that player unless the player puts a -1/-1 counter on a creature they control.",
        ),
    ],
    ..oracle::THELONS_CHANT
};
