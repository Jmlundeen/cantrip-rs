use cantrip_api::*;
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

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP.and(life(1)), add_any_color(1), "{T}, Pay 1 life: Add one mana of any color."),
    ],
    ..oracle::STARTING_TOWN
};
