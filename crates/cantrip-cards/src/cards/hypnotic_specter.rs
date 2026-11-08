use cantrip_api::*;
use cantrip_mechanics::discard_at_random;
use cantrip_oracle as oracle;

/// "Whenever this creature deals damage to an opponent, that player discards a card at random." (CR 608.2).
fn player_discards_card_random(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = item.subject.player() {
        discard_at_random(g, player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::DAMAGE, cantrip_mechanics::triggers::this_deals_damage_to_opponent),
            player_discards_card_random,
            "Whenever this creature deals damage to an opponent, that player discards a card at random.",
        ),
    ],
    ..oracle::HYPNOTIC_SPECTER
};
