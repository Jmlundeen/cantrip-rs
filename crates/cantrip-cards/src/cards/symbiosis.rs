use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Two target creatures each get +2/+2 until end of turn." Exactly two are chosen (CR 601.2c); each one still a legal target as the
/// spell resolves gets the bonus (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for slot in 0..item.target_count as usize {
        if let Some(creature) = targets::object(g, item, slot) {
            pump(g, item.object, creature, 2, 2);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE.between(2, 2)], effect), ..oracle::SYMBIOSIS };
