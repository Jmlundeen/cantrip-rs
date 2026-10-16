use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_oracle as oracle;

/// Put a hone counter on each Equipment you control: those `you` control as the ability resolves (CR 608.2h), whatever
/// they are attached to or not. Each counter gives the creature its Equipment is attached to +1/+0 (CR 122.1j, the layers).
fn hone_each_equipment(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let equipment: Vec<ObjectId> =
        g.permanents_of(you).into_iter().filter(|&id| g.chars(id).has_subtype(Subtype::Equipment)).collect();
    for id in equipment {
        g.add_counters(id, CounterKind::Hone, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS_OR_ATTACKS,
        hone_each_equipment,
        "Whenever Dwalin enters or attacks, put a hone counter on each Equipment you control. (Each hone counter on an Equipment grants +1/+0 to equipped creature.)",
    )],
    ..oracle::DWALIN_WEAPONMASTER
};
