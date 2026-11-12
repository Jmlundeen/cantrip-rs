use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{T}: Add X mana of any one color, where X is the greatest mana value among commanders you control." Only a creature a
/// game designated as a commander counts, and only while it is a permanent you control (CR 903.3, 109.4): a commander in the
/// command zone is controlled by nobody, so it adds nothing, and with no commander at all X is 0 and no mana is added.
fn add_greatest_commander_mana(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let you = asker.controller;
    let greatest = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.obj(id).controller == you && g.state().obj(id).commander)
        .map(|id| g.mana_value(id))
        .max()
        .unwrap_or_default();
    if greatest > 0 {
        let mana = color.map(Color::mana).unwrap_or(Mana::C);
        g.add_mana(you, mana, greatest as u32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(3), "{T}: Add three mana of any one color."),
        mana_ability(
            TAP,
            add_any_color(0).by(add_greatest_commander_mana),
            "{T}: Add X mana of any one color, where X is the greatest mana value among commanders you control.",
        ),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::COMMAND_CHAIR
};
