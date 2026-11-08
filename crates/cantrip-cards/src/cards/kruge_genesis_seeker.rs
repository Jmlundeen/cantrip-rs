use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ATTACKS;
use cantrip_oracle as oracle;

/// "Whenever Kruge attacks, attacking Klingons you control get +1/+1 until end of turn" (CR 508.3d, 611.2c): the attacking
/// Klingon creatures its controller controls as it resolves, Kruge included, and none that attack later.
fn pump_attacking_klingons(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let klingon = g.words(item.object).subtype(Subtype::Klingon);
    let attackers: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.is_attacking(id) && g.chars(id).has_subtype(klingon))
        .collect();
    instruction(g, |g| {
        for id in attackers {
            cantrip_mechanics::pump(g, item.object, id, 1, 1);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS,
        pump_attacking_klingons,
        "Whenever Kruge attacks, attacking Klingons you control get +1/+1 until end of turn.",
    )],
    ..oracle::KRUGE_GENESIS_SEEKER
};
