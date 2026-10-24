use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control, sub};
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "{2}{W}{W}, {T}: Create a 1/1 white Human creature token."
fn create_a_human(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::HUMAN, 1)?;
    Ok(())
}

/// "This land enters tapped unless you control a Plains." "{T}: Add {W}."
pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control::<{ sub(Subtype::Plains) }>),
    abilities: &[
        mana_ability(TAP, add(Mana::W), "{T}: Add {W}."),
        ability(cost("{2}{W}{W}, {T}"), create_a_human, "{2}{W}{W}, {T}: Create a 1/1 white Human creature token."),
    ],
    ..oracle::CASTLE_ARDENVALE
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN];
