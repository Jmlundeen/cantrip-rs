use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy target nonland permanent. Its controller creates a Lander token." (CR 701.8, 111.2.) The controller is read
/// before the destroy and is the one it last had (CR 608.2h), so the token is made whether or not it was destroyed.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(permanent);
    g.destroy(permanent);
    tokens::create_lander(g, controller)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], effect),
    ..oracle::EMERGENCY_EJECT
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDER];
