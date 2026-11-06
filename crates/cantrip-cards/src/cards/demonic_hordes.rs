use cantrip_api::*;
use cantrip_mechanics::{choose, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, unless you pay {B}{B}{B}, tap this creature and sacrifice a land of an opponent's choice"
/// (CR 118.12a, 701.21a, 608.2d): the controller may pay; if they do not, Demonic Hordes is tapped and one of the lands they
/// control, the one an opponent chooses (which opponent is the controller's choice), is sacrificed.
fn tap_and_sacrifice_a_land_unless_pay(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    static QUESTION: Question = Question::new(
        "Pay",
        "pay {cost}?",
        "If you don't, tap {subject} and sacrifice a land of an opponent's choice",
        "Pay {cost}",
        "Don't pay",
    )
    .no_default();
    if g.may_pay(item.controller, item.object, item.object, cost("{B}{B}{B}"), &QUESTION)? {
        return Ok(());
    }
    g.tap(item.object);
    let lands: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Land) && g.can_sacrifice(item.controller, id)).collect();
    if let Some(land) = choose::opponent_chooses_one(g, item.controller, item.object, Purpose::Choose, lands)? {
        g.sacrifice(item.controller, &[land]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::destroy_first_target, "{T}: Destroy target land.").targets(&[targets::LAND]),
        triggered(
            triggers::YOUR_UPKEEP,
            tap_and_sacrifice_a_land_unless_pay,
            "At the beginning of your upkeep, unless you pay {B}{B}{B}, tap this creature and sacrifice a land of an opponent's choice.",
        ),
    ],
    ..oracle::DEMONIC_HORDES
};
