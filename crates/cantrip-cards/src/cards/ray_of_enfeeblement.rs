use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets -4/-1 until end of turn. If that creature is white, it gets -4/-4 until end of turn instead."
/// The color is read as the spell resolves (CR 608.2h), through the words of the spell's text (CR 612); "instead" makes it
/// one effect, so a white creature gets only -4/-4 (CR 608.2c), never -4/-1 as well, until end of turn (CR 611.2a).
fn enfeeble(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        let white = g.chars(creature).colors.has(g.words(item.object).color(Color::White));
        let toughness = if white { -4 } else { -1 };
        pump(g, item.object, creature, -4, toughness);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], enfeeble), ..oracle::RAY_OF_ENFEEBLEMENT };
