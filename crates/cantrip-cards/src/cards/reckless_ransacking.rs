use cantrip_api::*;
use cantrip_mechanics::{pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+2 until end of turn. Create a Treasure token." With its one target illegal the whole spell
/// is countered (CR 608.2b), so no Treasure is made; otherwise both instructions are followed in order (CR 608.2c).
fn ransack(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, 3, 2);
    }
    tokens::create_a_treasure(g, item)
}

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], ransack),
    ..oracle::RECKLESS_RANSACKING
};
