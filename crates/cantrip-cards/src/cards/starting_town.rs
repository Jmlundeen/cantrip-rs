use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "Enters tapped unless it's your first, second, or third turn of the game": only the active player has a turn
/// going on, so on anyone else's turn it enters tapped.
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let player = entering.controller;
    if g.state().turn.active != player || g.state().player(player).turns_taken > 3 {
        entering.tapped = true;
    }
    Ok(())
}

fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Pay 1 life: Add one mana of any color.",
            cost: Cost { tap: true, life: 1, ..Cost::NONE },
            mana: Some(add_one_of_any_color),
            any_color: true,
            adds_any: 1,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::STARTING_TOWN
};
