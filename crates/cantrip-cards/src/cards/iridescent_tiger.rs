use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_IF_CAST;
use cantrip_oracle as oracle;

/// "Add {W}{U}{B}{R}{G}" from a triggered ability: not a mana ability (CR 605.1b), so it uses the stack and can be responded to,
/// and the mana empties at the end of the step (CR 500.5). It triggers only if the Tiger was cast, from any zone (CR 603.4).
fn add_one_mana_of_each_color(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for mana in [Mana::W, Mana::U, Mana::B, Mana::R, Mana::G] {
        g.add_mana(item.controller, mana, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS_IF_CAST, add_one_mana_of_each_color, "When this creature enters, if you cast it, add {W}{U}{B}{R}{G}.")],
    ..oracle::IRIDESCENT_TIGER
};
