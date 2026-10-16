use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Converge: "exile target nonland permanent if its mana value is less than or equal to the number of colors of mana
/// spent to cast this spell". Colorless is not a color (CR 105.1), and a spell that was not cast (a copy) spent nothing.
/// The comparison is made as it resolves, so a permanent that costs more is a legal target the spell does nothing to.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let spent = g.obj(item.object).spent;
    let colors = [Mana::W, Mana::U, Mana::B, Mana::R, Mana::G].into_iter().filter(|&m| spent.get(m) > 0).count() as u32;
    if g.mana_value(permanent) <= colors {
        g.exile(permanent);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effect),
    ..oracle::PRISMATIC_ENDING
};
