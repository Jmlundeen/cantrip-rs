use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers::LANDFALL, warp};
use cantrip_oracle as oracle;

/// "Landfall — Whenever a land you control enters, create a 2/2 colorless Robot artifact creature token." (CR 111.1)
fn robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_2_2, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        LANDFALL,
        robot,
        "Landfall \u{2014} Whenever a land you control enters, create a 2/2 colorless Robot artifact creature token.",
    )],
    ways: &[warp::way(mana("{1}{G}"), "Warp {1}{G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::EUSOCIAL_ENGINEERING
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_2_2];
