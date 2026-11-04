use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever this creature deals damage to a player, that player gets two poison counters." (CR 608.2).
fn player_gets_two_poison_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.subject.player() else { return Ok(()) };
    g.give_poison(player, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::DAMAGE, triggers::this_deals_damage_to_a_player),
            player_gets_two_poison_counters,
            "Whenever this creature deals damage to a player, that player gets two poison counters.",
        ),
    ],
    ..oracle::MARSH_VIPER
};
