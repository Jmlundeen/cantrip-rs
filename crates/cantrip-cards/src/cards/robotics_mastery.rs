use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::{attach::attached_object, targets, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Create two 1/1 colorless Robot artifact creature tokens with flying" (CR 111.1): both are made together (CR 101.4).
fn make_two_robots(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_1_1_FLYING, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(THIS_ENTERS, make_two_robots, "When this Aura enters, create two 1/1 colorless Robot artifact creature tokens with flying.")],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Enchanted creature gets +2/+2.")],
    ..oracle::ROBOTICS_MASTERY
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_1_1_FLYING];
