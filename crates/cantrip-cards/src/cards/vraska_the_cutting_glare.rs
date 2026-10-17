use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{destroy_target, lands_you_control, targets, tokens};
use cantrip_oracle as oracle;

/// The intervening "if you control six or more lands" (CR 603.4): checked as Vraska enters and again as the ability resolves.
fn six_or_more_lands(q: &dyn Query, asker: &Asker) -> bool {
    lands_you_control(q, asker.controller) >= 6
}

/// "Destroy target permanent an opponent controls. They create a Treasure token." (CR 701.8, 111.10a) "They" is the
/// target's controller as it resolves; they create the Treasure whether or not the destroy happened (indestructible,
/// regeneration). With the target illegal the ability does not resolve (CR 608.2b), so no Treasure is made.
fn destroy_and_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(target);
    destroy_target(g, item, 0);
    tokens::create_treasure(g, controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        destroy_and_treasure,
        "When Vraska enters, if you control six or more lands, destroy target permanent an opponent controls. They create a Treasure token.",
    )
    .condition(six_or_more_lands)
    .targets(&[targets::PERMANENT_AN_OPPONENT_CONTROLS])],
    ..oracle::VRASKA_THE_CUTTING_GLARE
};
