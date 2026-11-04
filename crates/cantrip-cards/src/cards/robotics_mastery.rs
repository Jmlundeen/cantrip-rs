use cantrip_api::*;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::{attach::attached_object, targets, tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// A 1/1 colorless Robot artifact creature token with flying (CR 111). Robotics Mastery creates two.
pub static ROBOT_FLYING: CardDef = CardDef {
    colors: Some(ColorSet::NONE),
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    keywords: Keywords::of(&[Keyword::Flying]),
    ..token!(Robot)
};

/// "Create two 1/1 colorless Robot artifact creature tokens with flying" (CR 111.1): both are made together (CR 101.4).
fn make_two_robots(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &ROBOT_FLYING, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(THIS_ENTERS, make_two_robots, "When this Aura enters, create two 1/1 colorless Robot artifact creature tokens with flying.")],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<2, 2>, "Enchanted creature gets +2/+2.")],
    ..oracle::ROBOTICS_MASTERY
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_FLYING];
