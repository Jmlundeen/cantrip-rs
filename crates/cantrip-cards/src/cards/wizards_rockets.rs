use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "Add X mana in any combination of colors" (CR 106.1b): X was announced with the activation (CR 107.3a, 605.3a) and
/// is kept on the source; each mana is a color chosen as the ability resolves. Nothing is asked for X of zero.
fn add_x_mana(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let x = g.obj(asker.source).x;
    if x == 0 {
        return Ok(());
    }
    let ask = RequestKind::ChooseColors { source: asker.source, choices: ColorSet(0b11111), count: x.min(255) as u8 };
    if let Answer::Colors(colors) = g.ask(asker.controller, ask)? {
        for color in colors {
            g.add_mana(asker.controller, color.mana(), 1);
        }
    }
    Ok(())
}

/// "When this artifact is put into a graveyard from the battlefield" (CR 700.4, 603.10a): looked back on.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(cost("{X}, {T}").and(SACRIFICE_THIS), add_by(add_x_mana), "{X}, {T}, Sacrifice this artifact: Add X mana in any combination of colors."),
        triggered(THIS_DIES, |g, item| effects::draw(g, item, 1), "When this artifact is put into a graveyard from the battlefield, draw a card.")
            .look_back(),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::WIZARDS_ROCKETS
};
