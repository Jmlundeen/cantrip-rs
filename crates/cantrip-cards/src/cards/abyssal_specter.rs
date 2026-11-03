use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, triggers};
use cantrip_oracle as oracle;

/// "Whenever this creature deals damage to a player, that player discards a card." (CR 608.2, 701.9a).
fn player_discards_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.subject.player() else { return Ok(()) };
    discard_chosen(g, player, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when(EventMask::DAMAGE, triggers::this_deals_damage_to_a_player),
            player_discards_card,
            "Whenever this creature deals damage to a player, that player discards a card.",
        ),
    ],
    ..oracle::ABYSSAL_SPECTER
};
