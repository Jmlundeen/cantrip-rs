use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This land enters tapped unless you control a planeswalker" A planeswalker entering simultaneously does not
/// count (CR 614.1c).
fn enters_tapped_unless_you_control_a_planeswalker(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let controls_planeswalker = g
        .permanents_of(entering.controller)
        .into_iter()
        .any(|id| g.obj(id).became.is_none() && g.chars(id).is(CardType::Planeswalker));
    if !controls_planeswalker {
        entering.tapped = true;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_a_planeswalker),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Black, Color::Green]), "{T}: Add {B} or {G}.")],
    ..oracle::FORMIDABLE_COMMONS
};
