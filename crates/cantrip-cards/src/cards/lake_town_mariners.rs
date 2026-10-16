use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::ward};
use cantrip_oracle as oracle;

/// "Creatures and/or lands you control": a creature or a land on the battlefield under the spell's controller.
fn creature_or_land_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    ((targets::CREATURE.filter)(q, asker, t) || (targets::LAND.filter)(q, asker, t))
        && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

/// "Exile two target creatures and/or lands you control, then return them to the battlefield under their owner's control." Both
/// are exiled before either returns (CR 608.2c); each returns as a new object (CR 400.7), a token exiled this way ceases to exist
/// and does not return (CR 111.7). Targets no longer legal as it resolves are left alone (CR 608.2b).
fn gone_fishing(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let chosen: Vec<ObjectId> = (0..item.target_count as usize).filter_map(|slot| targets::object(g, item, slot)).collect();
    cantrip_mechanics::exile_and_return(g, &chosen)
}

/// The Adventure (CR 715).
pub static GONE_FISHING: CardDef = CardDef {
    spell: spell(&[target(creature_or_land_you_control).between(2, 2)], gone_fishing),
    ..oracle::GONE_FISHING
};

pub static CARD: CardDef = CardDef {
    abilities: &[ward::<2>("Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)")],
    adventure: Some(&GONE_FISHING),
    ..oracle::LAKE_TOWN_MARINERS
};

pub static EXTRA: &[&CardDef] = &[&GONE_FISHING];
