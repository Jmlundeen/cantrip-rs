use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(
        effects::damage_first_target_by_creatures_you_control,
        "Coordinated Maneuver deals damage equal to the number of creatures you control to target creature or planeswalker.",
    )
    .targets(&[targets::CREATURE_OR_PLANESWALKER]),
    mode(effects::destroy_first_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::COORDINATED_MANEUVER };
