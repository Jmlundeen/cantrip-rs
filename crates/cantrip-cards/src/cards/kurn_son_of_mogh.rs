use cantrip_api::*;
use cantrip_mechanics::battlefront::opponents_attacked_this_turn;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Battlefront — Whenever Kurn attacks, you draw X cards and lose X life, where X is the number of opponents you
/// attacked this turn" (CR 508.3a, 121.1, 119.3): Kurn's controller draws the cards, then loses that much life.
fn draw_x_and_lose_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = opponents_attacked_this_turn(g, item.controller);
    effects::draw(g, item, x)?;
    g.lose_life(item.controller, x as i32);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        draw_x_and_lose_x,
        "Battlefront — Whenever Kurn attacks, you draw X cards and lose X life, where X is the number of opponents you attacked this turn.",
    )],
    ..oracle::KURN_SON_OF_MOGH
};
