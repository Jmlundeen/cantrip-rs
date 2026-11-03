use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// The creature attached as the ability resolves, or the Aura's last known attachment if it has left (CR 303.4m, 608.2h).
fn bless(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = g.obj(item.object).attached_to {
        pump(g, item.object, creature, 1, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{W}"), bless, "{W}: Enchanted creature gets +1/+1 until end of turn.")],
    enchant: Some(targets::CREATURE),
    ..oracle::BLESSING
};
