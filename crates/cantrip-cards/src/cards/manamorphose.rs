use cantrip_api::*;
use cantrip_oracle as oracle;

/// Add two mana in any combination of colors (CR 106.1b), the same color twice included: both chosen as the spell
/// resolves, in one question. Then draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for color in cantrip_mechanics::choose_colors(g, item.controller, item.object, ColorSet(0b11111), 2)? {
        g.add_mana(item.controller, color.mana(), 1);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::MANAMORPHOSE };
