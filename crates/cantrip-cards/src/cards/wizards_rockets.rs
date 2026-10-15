use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::this_dies;
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
        AbilityDef::Activated(ActivatedDef {
            text: "{X}, {T}, Sacrifice this artifact: Add X mana in any combination of colors.",
            cost: Cost { mana: ManaCost { x: 1, ..ManaCost::ZERO }, tap: true, sacrifice_self: true, ..Cost::NONE },
            mana: Some(add_x_mana),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact is put into a graveyard from the battlefield, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            effect: effects::draw::<1>,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::WIZARDS_ROCKETS
};
