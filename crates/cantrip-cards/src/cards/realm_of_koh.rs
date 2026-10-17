use cantrip_api::*;
use cantrip_mechanics::lands::enters_tapped_unless_you_control_basic_land;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Create a 1/1 colorless Spirit creature token with 'This token can't block or be blocked by non-Spirit creatures.'"
fn create_spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::SPIRIT, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_basic_land),
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        ability(
            cost("{3}{B}, {T}"),
            create_spirit,
            "{3}{B}, {T}: Create a 1/1 colorless Spirit creature token with \"This token can't block or be blocked by non-Spirit creatures.\"",
        ),
    ],
    ..oracle::REALM_OF_KOH
};

pub static EXTRA: &[&CardDef] = &[&tokens::SPIRIT];
