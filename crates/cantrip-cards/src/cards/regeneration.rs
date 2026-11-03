use cantrip_api::*;
use cantrip_mechanics::{regenerate, targets};
use cantrip_oracle as oracle;

/// The enchanted creature when this ability resolves, or the Aura's last known attachment if it has left (CR 303.4m, 608.2h).
fn regenerate_enchanted_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = g.obj(item.object).attached_to {
        regenerate::permanent(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{G}"), regenerate_enchanted_creature, "{G}: Regenerate enchanted creature.")],
    enchant: Some(targets::CREATURE),
    ..oracle::REGENERATION
};
