use cantrip_api::*;
use cantrip_mechanics::{
    attach::{attached_to_it, tap_attached},
    effects, targets,
    triggers::{ENCHANTED_CREATURE_DEALT_DAMAGE, THIS_ENTERS},
};
use cantrip_oracle as oracle;

/// "When this Aura enters, tap enchanted creature." The creature it is attached to as the trigger resolves (CR 303.4m, 608.2h).
fn tap_enchanted(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tap_attached(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    abilities: &[
        triggered(THIS_ENTERS, tap_enchanted, "When this Aura enters, tap enchanted creature."),
        triggered(ENCHANTED_CREATURE_DEALT_DAMAGE, effects::sacrifice_self, "When enchanted creature is dealt damage, sacrifice this Aura.")
            .sums_per_subject(),
    ],
    untap_bans: &[attached_to_it],
    ..oracle::SLEEP_MAGIC
};
