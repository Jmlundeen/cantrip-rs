use cantrip_api::*;
use cantrip_mechanics::{players, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, for each player, this enchantment deals 1 damage to that player unless they pay {B} or {3}." (CR 608.2, 120.3, 118.1, 800.4a).
fn player_enchantment_deals_1_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let mut unpaid: Vec<(ObjectId, Target, u32)> = Vec::new();
    for p in players {
        if !players::reached(g, p) {
            continue;
        }
        let paid = g.may_pay(p, item.object, item.object, cost("{B}"), &PAY_BLACK)?
            || g.may_pay(p, item.object, item.object, cost("{3}"), &PAY_THREE)?;
        if !paid {
            unpaid.push((item.object, Target::Player(p), 1));
        }
    }
    g.end_sentence();
    g.damage_together(&unpaid);
    Ok(())
}

static PAY_BLACK: Question = Question::new(
    "Pay",
    "pay {cost}, or choose whether to pay {3} instead?",
    "",
    "Pay {cost}",
    "Consider paying {3}",
);

static PAY_THREE: Question = Question::new(
    "Pay",
    "pay {cost}, or take 1 damage?",
    "",
    "Pay {cost}",
    "Take 1 damage",
);

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            player_enchantment_deals_1_damage,
            "At the beginning of your upkeep, for each player, this enchantment deals 1 damage to that player unless they pay {B} or {3}.",
        ),
    ],
    ..oracle::LIM_D_LS_HEX
};
