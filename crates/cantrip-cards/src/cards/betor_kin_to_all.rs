use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "Creatures you control have total toughness N or greater" (CR 208.1, 107.1b): the toughnesses as they are now, a negative one
/// taking away from the total.
fn total_toughness(q: &dyn Query, player: PlayerId) -> i32 {
    q.permanents_of(player)
        .into_iter()
        .filter(|&id| q.chars(id).is(CardType::Creature))
        .filter_map(|id| q.chars(id).toughness)
        .map(i32::from)
        .sum()
}

/// "If creatures you control have total toughness 10 or greater" (CR 603.4): an intervening if, asked as the end step begins (below
/// 10 the ability does not trigger at all) and again as it resolves (below 10 then, none of it happens), per its ruling.
fn total_toughness_ten_or_greater(q: &dyn Query, asker: &Asker) -> bool {
    total_toughness(q, asker.controller) >= 10
}

/// "Draw a card. Then if creatures you control have total toughness 20 or greater, untap each creature you control. Then if
/// creatures you control have total toughness 40 or greater, each opponent loses half their life, rounded up." (CR 608.2c, 119.3)
/// Each later "if" is checked as its step is reached, not once at the start.
fn bigger_and_bigger(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = item.controller;
    effects::draw(g, item, 1)?;
    if total_toughness(g, me) >= 20 {
        let creatures: Vec<ObjectId> = g.permanents_of(me).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
        for creature in creatures {
            g.untap(creature);
        }
    }
    if total_toughness(g, me) >= 40 {
        // Every opponent's half is worked out first, then they all lose it together (CR 101.4, 119.3).
        let losses: Vec<(PlayerId, i32)> = g.state().opponents(me).map(|opponent| (opponent, (g.state().player(opponent).life.max(0) + 1) / 2)).collect();
        g.lose_life_together(&losses);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        bigger_and_bigger,
        "At the beginning of your end step, if creatures you control have total toughness 10 or greater, draw a card. Then if creatures you control have total toughness 20 or greater, untap each creature you control. Then if creatures you control have total toughness 40 or greater, each opponent loses half their life, rounded up.",
    )
    .condition(total_toughness_ten_or_greater)],
    ..oracle::BETOR_KIN_TO_ALL
};
