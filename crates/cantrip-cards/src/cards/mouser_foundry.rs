use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers::THIS_ENTERS_OR_LEAVES};
use cantrip_oracle as oracle;

fn make_a_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ROBOT_1_1, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_LEAVES, make_a_robot, "When this artifact enters or leaves the battlefield, create a 1/1 colorless Robot artifact creature token."),
        ability(
            cost("{4}{R}").and(SACRIFICE_THIS),
            effects::damage_first_target::<3>,
            "{4}{R}, Sacrifice this artifact: It deals 3 damage to target creature.",
        )
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::MOUSER_FOUNDRY
};

pub static EXTRA: &[&CardDef] = &[&tokens::ROBOT_1_1];
