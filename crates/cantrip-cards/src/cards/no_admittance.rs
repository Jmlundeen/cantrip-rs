use cantrip_api::*;
use cantrip_mechanics::{damage_target, empower, targets};
use cantrip_oracle as oracle;

/// "No Admittance deals 3 damage to any target. Empower Jace 1." The damage first, then the empower, both as the spell
/// resolves (CR 608.2c); damage to a target that has become illegal is not dealt (CR 608.2b).
fn no_admittance(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    empower::jace(g, item.controller, item.object, 1)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], no_admittance),
    ..oracle::NO_ADMITTANCE
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
