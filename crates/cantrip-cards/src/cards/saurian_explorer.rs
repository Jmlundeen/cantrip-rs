use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// "Create a tapped Planet land token named New Planet with '{T}: Add one mana of any color.'" (CR 111.1, 111.4).
fn create_new_planet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_by_rule(g, item.controller, &tokens::NEW_PLANET, 1, true)?;
    Ok(())
}

/// "Landfall — Whenever a land you control enters, this creature gets +2/+2 until end of turn" (CR 611.2a, 603.6a), and
/// "{6}: Create a tapped Planet land token named New Planet with '{T}: Add one mana of any color.' Activate only as a
/// sorcery." (CR 602.5d, 701.7).
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            LANDFALL,
            effects::this_gets::<2, 2>,
            "Landfall \u{2014} Whenever a land you control enters, this creature gets +2/+2 until end of turn.",
        ),
        ability(
            cost("{6}"),
            create_new_planet,
            "{6}: Create a tapped Planet land token named New Planet with \"{T}: Add one mana of any color.\" Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::SAURIAN_EXPLORER
};

pub static EXTRA: &[&CardDef] = &[&tokens::NEW_PLANET];
