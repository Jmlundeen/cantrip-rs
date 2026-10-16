use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::tokens::ROBOT_2_2;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "You may discard a card. If you do, draw two cards." Choosing no card declines; the draw follows a card that was
/// actually discarded. Nothing is asked of an empty hand.
fn discard_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    if let Some(&card) = choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?.first() {
        g.discard(card);
        g.draw(item.controller, 2);
    }
    Ok(())
}

fn make_robot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let robot = g.card_id(&ROBOT_2_2);
    g.create_tokens(robot, item.controller, 1, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, discard_to_draw, "When Melded Moxite enters, you may discard a card. If you do, draw two cards."),
        ability(cost("{3}").and(SACRIFICE_THIS), make_robot, "{3}, Sacrifice Melded Moxite: Create a tapped 2/2 colorless Robot artifact creature token."),
    ],
    ..oracle::MELDED_MOXITE
};

pub static EXTRA: &[&CardDef] = &[&ROBOT_2_2];
