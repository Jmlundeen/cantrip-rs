use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::ATTACHED_CREATURE_DIES_ABOUT_IT;
use cantrip_oracle as oracle;

/// "This Aura deals damage equal to that creature's toughness to the creature's controller" (CR 608.2h): the creature, the
/// ability's subject, as it last existed on the battlefield - its toughness with every effect on it then, and the player who
/// then controlled it. A toughness less than 0 is counted as 0 (CR 107.1b), and 0 damage is no damage (CR 120.8).
fn damage_its_controller(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    let toughness = g.chars(creature).toughness.unwrap_or(0).max(0) as u32;
    let controller = g.controller(creature);
    g.damage(item.object, Target::Player(controller), toughness);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[triggered(
        ATTACHED_CREATURE_DIES_ABOUT_IT,
        damage_its_controller,
        "When enchanted creature dies, this Aura deals damage equal to that creature's toughness to the creature's controller.",
    )],
    ..oracle::CREATURE_BOND
};
