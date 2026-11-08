use cantrip_api::*;
use cantrip_mechanics::sunburst;
use cantrip_oracle as oracle;

/// Converge: "You draw X cards and lose X life, where X is the number of colors of mana spent to cast this spell."
/// (CR 105.1, 207.2c, 121.1, 119.3): X counts distinct colors spent, colorless is not a color; a copy was never cast, so X is
/// zero (`sunburst::colors_spent`).
fn draw_x_lose_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = sunburst::colors_spent(g, item.object);
    g.draw(item.controller, x as u32);
    g.lose_life(item.controller, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], draw_x_lose_x),
    ..oracle::PAINFUL_TRUTHS
};
