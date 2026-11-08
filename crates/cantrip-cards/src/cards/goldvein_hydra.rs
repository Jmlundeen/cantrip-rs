use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "When this creature dies, create a number of tapped Treasure tokens equal to its power." Its power as it last existed on the
/// battlefield (CR 608.2h, the ruling): the +1/+1 counters and any pump or other effect it had as it died. The tokens are made
/// together (CR 101.4) under the ability's controller, none for a power of 0 or less (CR 107.1b).
fn create_treasures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_equal_to_last_known_power(g, item, &tokens::TREASURE, true)
}

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::effects::enters_with_x_counters::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(THIS_DIES, create_treasures, "When this creature dies, create a number of tapped Treasure tokens equal to its power.")],
    ..oracle::GOLDVEIN_HYDRA
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
