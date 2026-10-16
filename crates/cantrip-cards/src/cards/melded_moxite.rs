use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::tokens::ROBOT_2_2;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn make_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let robot = g.card_id(&ROBOT_2_2);
    g.create_tokens(robot, item.controller, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::may_discard_to_draw(g, item, 2), "When Melded Moxite enters, you may discard a card. If you do, draw two cards."),
        ability(cost("{3}").and(SACRIFICE_THIS), make_robot, "{3}, Sacrifice Melded Moxite: Create a tapped 2/2 colorless Robot artifact creature token."),
    ],
    ..oracle::MELDED_MOXITE
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
