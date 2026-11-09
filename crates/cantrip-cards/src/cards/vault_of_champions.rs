use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This land enters tapped unless you have two or more opponents." (CR 614.1d): judged as it enters, so in a duel it
/// is tapped and in a multiplayer game it is not; a player eliminated before it enters is no longer an opponent.
/// "{T}: Add {W} or {B}" is an ordinary two-color mana ability (CR 605.1a).
fn enters_tapped_unless_two_or_more_opponents(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    if g.state().opponents(entering.controller).count() < 2 {
        entering.tapped = true;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_two_or_more_opponents),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Black]), "{T}: Add {W} or {B}.")],
    ..oracle::VAULT_OF_CHAMPIONS
};
