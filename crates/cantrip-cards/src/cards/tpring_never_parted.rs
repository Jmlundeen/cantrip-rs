use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, fight, targets, triggers};
use cantrip_oracle as oracle;

const MAY_FIGHT: Question = Question::new("KalIfFee", "have the chosen creatures fight each other?", "", "Fight", "Don't fight");

/// "Kal-if-fee — When T'Pring enters, choose another target creature you control and target creature an opponent
/// controls. You may have the chosen creatures fight each other. Whenever a creature that fought this way dies this
/// turn, you draw a card." (CR 701.14a-b, 603.7a-c, 608.2d): the fight happens only if the controller says so, and then
/// a delayed triggered ability watches those two objects for the rest of the turn - a creature that leaves and returns is
/// a new object and is not watched (CR 400.7). Declining the fight sets up nothing, and a creature of the pair that
/// stops being a creature or is gone as the ability resolves means neither fights (CR 701.14b).
fn kal_if_fee(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let fought = targets::legal_objects(g, item);
    if fought.len() == 2 && effects::ask_may(g, item.controller, item.object, Purpose::Ask(&MAY_FIGHT))? {
        g.end_sentence();
        for creature in fought {
            delayed::this_turn_watching(g, item, creature);
        }
        fight::targets_fight(g, item, 0, 1);
    }
    Ok(())
}

/// "Whenever a creature that fought this way dies this turn, you draw a card" (CR 603.7c, 121.1): one card for each of
/// the watched creatures that dies, its controller drawing, whoever's turn it was.
fn draw_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        kal_if_fee,
        "Kal-if-fee \u{2014} When T'Pring enters, choose another target creature you control and target creature an opponent controls. You may have the chosen creatures fight each other. Whenever a creature that fought this way dies this turn, you draw a card.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS])],
    delayed: &[triggered(
        triggers::THIS_DIES,
        draw_one,
        "Whenever a creature that fought this way dies this turn, you draw a card.",
    )],
    ..oracle::TPRING_NEVER_PARTED
};
