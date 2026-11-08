use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "the number of opponents you attacked this turn" (CR 508.1): each player other than Kurn's controller that one of
/// their creatures was declared attacking this turn, counted once however many creatures attacked them; a planeswalker
/// or battle attacked is no player, and a creature put onto the battlefield attacking was never declared (CR 508.4).
fn opponents_attacked_this_turn(g: &dyn Game, you: PlayerId) -> usize {
    let mut attacked = Vec::new();
    for r in g.history().this_turn().iter().filter(|r| r.active == you) {
        if let Event::Attacks { defender: Target::Player(p), .. } = r.event
            && p != you
            && !attacked.contains(&p)
        {
            attacked.push(p);
        }
    }
    attacked.len()
}

/// "Battlefront — Whenever Kurn attacks, you draw X cards and lose X life, where X is the number of opponents you
/// attacked this turn" (CR 508.3d, 121.1, 119.3): Kurn's controller draws the cards, then loses that much life.
fn draw_x_and_lose_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = opponents_attacked_this_turn(g, item.controller) as u32;
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
