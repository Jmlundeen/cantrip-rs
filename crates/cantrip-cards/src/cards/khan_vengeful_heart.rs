use cantrip_api::*;
use cantrip_mechanics::{effects, select, triggers};
use cantrip_oracle as oracle;

/// "When Khan enters, you may sacrifice another creature or artifact. If you do, draw two cards." (CR 603.6a, 608.2d):
/// declining is the empty answer to an up-to-one choice, nothing is asked when Khan is the only permanent its controller
/// has (CR 701.21a), and the cards are drawn only for a permanent actually sacrificed (CR 608.2c). "Another" is any
/// permanent but Khan himself, of either type.
fn sacrifice_another_to_draw_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let rule = select::another_creature_or_artifact_permanents(select::up_to(1));
    if !effects::may_sacrifice(g, you, item.object, rule)?.is_empty() {
        effects::draw(g, item, 2)?;
    }
    Ok(())
}

/// "You may cast this card from exile" is a static ability that functions in exile (CR 113.6e). The engine has no route
/// for it yet: a card's permission to be cast from exile is `CardDef::exile_casts`, read off permanents on the
/// battlefield alone (`cantrip-rules`' `given`). Scenarios/khan-vengeful-heart-may-be-cast-from-exile.cantrip holds the
/// rules-correct outcome as a known bug.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        sacrifice_another_to_draw_two,
        "When Khan enters, you may sacrifice another creature or artifact. If you do, draw two cards.",
    )],
    ..oracle::KHAN_VENGEFUL_HEART
};
