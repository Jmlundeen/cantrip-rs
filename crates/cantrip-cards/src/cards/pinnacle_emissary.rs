use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::warp;
use cantrip_oracle as oracle;

/// Create a 1/1 colorless Drone artifact creature token with flying and "This token can block only creatures with flying."
fn drone(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::DRONE, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::ARTIFACT_SPELL_YOU_CAST, drone, "Whenever you cast an artifact spell, create a 1/1 colorless Drone artifact creature token with flying and \"This token can block only creatures with flying.\"")],
    ways: &[warp::way(mana("{U/R}"), "Warp {U/R}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::PINNACLE_EMISSARY
};

pub static EXTRA: &[&CardDef] = &[&tokens::DRONE];
