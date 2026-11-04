use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "You and target opponent each flip a coin. Mana Clash deals 1 damage to each player whose coin comes up tails. Repeat this process until both players' coins come up heads on the same flip." (CR 608.2, 120.3).
fn target_opponent_flip_coin_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    loop {
        let mine = g.flip_coin_for_heads(item.controller);
        let theirs = g.flip_coin_for_heads(opponent);
        g.end_sentence();
        let tails: Vec<(ObjectId, Target, u32)> = g
            .state()
            .apnap()
            .filter(|&p| if p == item.controller { !mine } else { p == opponent && !theirs })
            .map(|p| (item.object, Target::Player(p), 1))
            .collect();
        if tails.is_empty() {
            break;
        }
        g.damage_together(&tails);
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::OPPONENT],
        target_opponent_flip_coin_mana,
    ),
    ..oracle::MANA_CLASH
};
