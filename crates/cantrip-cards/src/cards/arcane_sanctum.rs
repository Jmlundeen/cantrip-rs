use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This land enters tapped. {T}: Add {W}, {U}, or {B}." (CR 614.1d).
fn add_w_u_b_enters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::White, Color::Blue, Color::Black]),
            "{T}: Add {W}, {U}, or {B}.",
        ),
    ],
    enters: Some(add_w_u_b_enters),
    ..oracle::ARCANE_SANCTUM
};
