use cantrip_api::*;
use cantrip_mechanics::tokens::{self, PILOT_SADDLES_AND_CREWS};
use cantrip_mechanics::effects::gain_hexproof_and_indestructible_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target permanent you control gains hexproof and indestructible until end of turn": from this spell's resolution
/// (CR 611.2a), and nothing if the target is no longer legal (CR 608.2b).
fn protect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        gain_hexproof_and_indestructible_until_end_of_turn(g, item.object, permanent);
    }
    Ok(())
}

/// "Create a 1/1 colorless Pilot creature token with \"This token saddles Mounts and crews Vehicles as though its power
/// were 2 greater.\""
fn pilot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &PILOT_SADDLES_AND_CREWS, 1)?;
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(protect, "Target permanent you control gains hexproof and indestructible until end of turn.").targets(&[targets::PERMANENT_YOU_CONTROL]),
    mode(pilot, "Create a 1/1 colorless Pilot creature token with \"This token saddles Mounts and crews Vehicles as though its power were 2 greater.\""),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::DEFEND_THE_RIDER };

pub static EXTRA: &[&CardDef] = &[&PILOT_SADDLES_AND_CREWS];
