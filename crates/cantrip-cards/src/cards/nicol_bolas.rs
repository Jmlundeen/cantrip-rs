use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, sacrifice Nicol Bolas unless you pay {U}{B}{R}." (CR 608.2, 701.21a, 118.1).
fn sacrifice_nicol_bolas_unless_pay(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::sacrifice_this_unless_pay(g, item, cost("{U}{B}{R}"))
}

/// "That player discards their hand" (CR 701.9a).
fn player_discards_their_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = item.subject.player() {
        g.discard_hand(player);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            sacrifice_nicol_bolas_unless_pay,
            "At the beginning of your upkeep, sacrifice Nicol Bolas unless you pay {U}{B}{R}.",
        ),
        triggered(
            when_asked(EventMask::DAMAGE, triggers::this_deals_damage_to_opponent),
            player_discards_their_hand,
            "Whenever Nicol Bolas deals damage to an opponent, that player discards their hand.",
        ),
    ],
    ..oracle::NICOL_BOLAS
};
