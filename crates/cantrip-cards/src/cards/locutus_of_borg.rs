use cantrip_api::*;
use cantrip_mechanics::statics::{grant_delayed_ability, your_artifact_creatures};
use cantrip_mechanics::tokens::{self, BORG};
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

/// "Whenever this creature deals combat damage to a player, create a 1/1 colorless Borg artifact creature token."
/// (CR 608.2d, 111.1): its controller gets the shared Borg token.
fn create_borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &BORG, 1)?;
    Ok(())
}

/// "Artifact creatures you control have [the trigger]" (CR 613.1f, 603.2): a layer 6 grant judged on what layers 1-5
/// made each permanent, so a creature that becomes an artifact creature has it, and an opponent's never does.
pub static CARD: CardDef = CardDef {
    delayed: &[triggered(
        DEALS_COMBAT_DAMAGE_TO_PLAYER,
        create_borg,
        "Whenever this creature deals combat damage to a player, create a 1/1 colorless Borg artifact creature token.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        your_artifact_creatures,
        grant_delayed_ability,
        "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, create a 1/1 colorless Borg artifact creature token.\"",
    )],
    ..oracle::LOCUTUS_OF_BORG
};

pub static EXTRA: &[&CardDef] = &[&BORG];
