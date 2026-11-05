use cantrip_api::*;
use cantrip_mechanics::triggers::creature_you_control_with_a_counter_deals_combat_damage_to_player;
use cantrip_oracle as oracle;

/// "Whenever a creature you control with a counter on it deals combat damage to a player, draw a card." (CR 510.2, 603.2,
/// 121.1): the ability's controller draws, however much damage was dealt and whatever creature dealt it.
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::DAMAGE, creature_you_control_with_a_counter_deals_combat_damage_to_player),
        draw_a_card,
        "Whenever a creature you control with a counter on it deals combat damage to a player, draw a card.",
    )],
    ..oracle::PHLOX_UNORTHODOX_DOCTOR
};
