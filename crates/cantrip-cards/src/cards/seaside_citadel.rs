use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This land enters tapped. {T}: Add {G}, {W}, or {U}." (CR 614.1d, 605.1a): a local enters fn so the enters-tapped
/// scenario executes this file's lines; the {T} ability is a mana ability offering green, white or blue and no other color.
fn add_g_w_u_enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        TAP,
        add_one_of(&[Color::Green, Color::White, Color::Blue]),
        "{T}: Add {G}, {W}, or {U}.",
    )],
    enters: Some(add_g_w_u_enters),
    ..oracle::SEASIDE_CITADEL
};
