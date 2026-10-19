use cantrip_api::*;
use cantrip_mechanics::amass::amass;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "When Azog enters, destroy up to one other target creature. Its controller amasses Goblins X, where X is that creature's
/// power. If you controlled that creature, draw a card." With no target, or one that is illegal on resolution, none of it
/// happens: "its controller" is undefined (CR 608.2b, the ruling). Who controlled the creature is read before it is destroyed
/// (CR 608.2h); a creature that survives (indestructible, regeneration) still has its controller amass and still draws you a
/// card. X is its power as it last existed on the battlefield (CR 608.2h), none if negative (CR 107.1b). The controller
/// amasses Goblins (CR 701.47a) with the word of Azog's own text (CR 612).
fn destroy_amass_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(creature);
    g.destroy(creature);
    let x = g.chars(creature).power.unwrap_or(0).max(0);
    amass(g, controller, item.object, &tokens::GOBLIN_ARMY, Subtype::Goblin, x)?;
    if controller == item.controller {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        destroy_amass_and_draw,
        "When Azog enters, destroy up to one other target creature. Its controller amasses Goblins X, where X is that creature's power. If you controlled that creature, draw a card.",
    )
    .targets(&[up_to_one(targets::other_creature_filter)])],
    ..oracle::AZOG_MORIAS_RUIN
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOBLIN_ARMY];
