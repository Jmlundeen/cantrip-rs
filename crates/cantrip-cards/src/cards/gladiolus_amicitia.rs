use cantrip_api::*;
use cantrip_mechanics::triggers::{LANDFALL, THIS_ENTERS};
use cantrip_mechanics::{library, pump, select, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Search your library for a land card, put it onto the battlefield tapped, then shuffle" (CR 701.23).
fn fetch_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::LAND_CARD, true)?;
    Ok(())
}

/// "Another target creature you control gets +2/+2 and gains trample until end of turn."
fn landfall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 2, 2);
        trample_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            fetch_land,
            "When Gladiolus Amicitia enters, search your library for a land card, put it onto the battlefield tapped, then shuffle.",
        ),
        triggered(
            LANDFALL,
            landfall,
            "Landfall \u{2014} Whenever a land you control enters, another target creature you control gets +2/+2 and gains trample until end of turn.",
        )
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL]),
    ],
    ..oracle::GLADIOLUS_AMICITIA
};
