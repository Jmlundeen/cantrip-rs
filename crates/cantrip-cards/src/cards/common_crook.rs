use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// "When this creature dies, create a Treasure token." The token is made for the ability's controller, who controlled the
/// creature as it died (CR 603.3a, 603.10a).
fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_a_treasure(g, item)
}

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, treasure, "When this creature dies, create a Treasure token.")],
    ..oracle::COMMON_CROOK
};
