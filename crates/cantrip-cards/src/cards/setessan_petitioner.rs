use cantrip_api::*;
use cantrip_mechanics::{devotion::devotion, effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, you gain life equal to your devotion to green." Devotion (CR 700.5) is counted as the
/// ability resolves (this creature itself counts if it is still on the battlefield), green read through the words of
/// this card's text (CR 612).
fn gain_life_equal_to_devotion(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = g.words(item.object).color(Color::Green);
    let n = devotion(g, item.controller, color) as i32;
    effects::gain_life(g, item, n)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, gain_life_equal_to_devotion, "When this creature enters, you gain life equal to your devotion to green.")],
    ..oracle::SETESSAN_PETITIONER
};
