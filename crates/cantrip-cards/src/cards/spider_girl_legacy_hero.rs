use cantrip_api::*;
use cantrip_mechanics::tokens::{self, HUMAN_CITIZEN};
use cantrip_mechanics::triggers::THIS_LEAVES;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "Create a 1/1 green and white Human Citizen creature token." Its controller is the one who controlled Spider-Girl as she left (CR 603.10a).
fn citizen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &HUMAN_CITIZEN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_flying, "During your turn, Spider-Girl has flying.")],
    abilities: &[triggered(THIS_LEAVES, citizen, "When Spider-Girl leaves the battlefield, create a 1/1 green and white Human Citizen creature token.")],
    ..oracle::SPIDER_GIRL_LEGACY_HERO
};

pub static EXTRA: &[&CardDef] = &[&HUMAN_CITIZEN];
