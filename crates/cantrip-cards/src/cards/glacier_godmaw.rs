use cantrip_api::*;
use cantrip_mechanics::tokens::{self, LANDER};
use cantrip_mechanics::triggers::{LANDFALL, THIS_ENTERS};
use cantrip_mechanics::{effects, haste_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Creatures you control get +1/+1 and gain vigilance and haste until end of turn" (CR 611.2, 611.2c): the creatures there are
/// as the trigger resolves, and no creature that enters later gets any of it.
fn pump_and_gain_vigilance_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    instruction(g, |g| {
        for id in effects::pump_creatures_you_control(g, item, 1, 1) {
            effects::gain_until_end_of_turn(g, item.object, id, Keyword::Vigilance);
            haste_until_end_of_turn(g, item.object, id);
        }
    });
    Ok(())
}

/// "When this creature enters, create a Lander token." (CR 111.10u)
fn lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, lander, "When this creature enters, create a Lander token."),
        triggered(
            LANDFALL,
            pump_and_gain_vigilance_and_haste,
            "Landfall \u{2014} Whenever a land you control enters, creatures you control get +1/+1 and gain vigilance and haste until end of turn.",
        ),
    ],
    ..oracle::GLACIER_GODMAW
};

pub static EXTRA: &[&CardDef] = &[&LANDER];
