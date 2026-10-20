use cantrip_api::*;
use cantrip_mechanics::devotion::devotion;
use cantrip_oracle as oracle;

/// "Add an amount of mana of that color equal to your devotion to that color" (CR 700.5): the color is named as
/// the ability is activated, the devotion counted as it resolves, after its {2} is paid (a mana ability, CR 605.3b).
fn add_devotion(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let Some(color) = color else { return Ok(()) };
    let amount = devotion(g, asker.controller, color);
    if amount > 0 {
        g.add_mana(asker.controller, color.mana(), amount.min(u8::MAX as u32) as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(cost("{2}, {T}"), add_any_color(1).by(add_devotion), "{2}, {T}: Choose a color. Add an amount of mana of that color equal to your devotion to that color."),
    ],
    ..oracle::NYKTHOS_SHRINE_TO_NYX
};
