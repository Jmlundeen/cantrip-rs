use cantrip_api::*;
use cantrip_mechanics::statics::{grant_delayed_ability, your_artifact_creatures};
use cantrip_mechanics::tokens::{self, BORG};
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER;
use cantrip_oracle as oracle;

const TEXT: &str = "Whenever this creature deals combat damage to a player, you may pay {1}. If you do, create a 1/1 colorless Borg artifact creature token.";

const BORG_Q: Question = Question::new("CreateToken", "pay {cost} to create a 1/1 Borg artifact creature token?", "", "Pay {cost}", "Don't pay");

/// "You may pay {1}. If you do, create a 1/1 colorless Borg artifact creature token." (CR 118.12, 608.2d, 111.1): the
/// controller of the ability, which is the controller of the creature that has it, pays and gets the token.
fn pay_for_borg(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{1}"), &BORG_Q)? {
        tokens::create(g, item.controller, &BORG, 1)?;
    }
    Ok(())
}

/// "Artifact creatures you control have [the trigger]" (CR 613.1f, 603.2): a layer 6 grant judged on what layers 1-5
/// made each permanent, so a creature that becomes an artifact creature has it, and an opponent's never does.
pub static CARD: CardDef = CardDef {
    delayed: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, pay_for_borg, TEXT)],
    statics: &[static_ability(
        Layer::Ability,
        your_artifact_creatures,
        grant_delayed_ability,
        "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, you may pay {1}. If you do, create a 1/1 colorless Borg artifact creature token.\"",
    )],
    ..oracle::ANTAGONIST_INCORPORATOR
};

pub static EXTRA: &[&CardDef] = &[&BORG];
