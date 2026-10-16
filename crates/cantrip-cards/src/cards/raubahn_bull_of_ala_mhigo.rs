use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, ward_life_equal_to_power};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const WARD: AbilityDef = ward_life_equal_to_power("Ward\u{2014}Pay life equal to Raubahn's power. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays life equal to its power.)");

/// "Attach up to one target Equipment you control to target attacking creature" (CR 701.3a): if both targets are still
/// legal, the Equipment is attached to the creature, and does nothing if it could not be attached there (CR 701.3b, the ruling:
/// "the Equipment won't move"). With no Equipment chosen, nothing happens. Either target having become illegal leaves the
/// attach undone (CR 608.2b).
fn attach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(equipment) = targets::object(g, item, 0)
        && let Some(creature) = targets::object(g, item, 1)
    {
        g.attach(equipment, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        WARD,
        triggered(THIS_ATTACKS, attach, "Whenever Raubahn attacks, attach up to one target Equipment you control to target attacking creature.")
            .targets(&[
                up_to_one(targets::EQUIPMENT_YOU_CONTROL.filter).labelled("Equipment to attach"),
                targets::ATTACKING_CREATURE.labelled("Attacking creature to attach it to"),
            ]),
    ],
    ..oracle::RAUBAHN_BULL_OF_ALA_MHIGO
};
