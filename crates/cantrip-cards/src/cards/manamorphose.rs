use cantrip_api::*;
use cantrip_oracle as oracle;

/// Add two mana in any combination of colors (CR 106.1b), the same color twice included: both chosen as the spell
/// resolves, in one question. Then draw a card.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let ask = RequestKind::ChooseColors { source: item.object, choices: ColorSet(0b11111), count: 2 };
    if let Answer::Colors(colors) = g.ask(item.controller, ask)? {
        for color in colors {
            g.add_mana(item.controller, color.mana(), 1);
        }
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::MANAMORPHOSE };
