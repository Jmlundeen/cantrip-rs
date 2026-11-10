use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever this creature attacks, add one mana of any color. Until end of turn, you don't lose this mana as steps and
/// phases end" (CR 106.3, 106.4, 514.2): one mana of the chosen color, added until end of turn rather than lost as the
/// steps before the turn's end pass.
fn add_one_mana_any_color_until_end_of_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let color = cantrip_mechanics::choose_color_for(g, item.controller, item.object, ColorUse::AddMana)?;
    g.add_mana_until_end_of_turn(item.controller, color.mana(), 1);
    Ok(())
}

// Menace is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        add_one_mana_any_color_until_end_of_turn,
        "Whenever this creature attacks, add one mana of any color. Until end of turn, you don't lose this mana as steps and phases end.",
    )],
    ..oracle::QONOS_QUARTERMASTER
};
