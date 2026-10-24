use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Destroy target creature with toughness 4 or greater." / "Destroy target enchantment." (CR 701.8): each mode's one target.
static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target creature with toughness 4 or greater.")
        .targets(&[targets::CREATURE_WITH_TOUGHNESS_4_OR_GREATER]),
    mode(effects::destroy_first_target, "Destroy target enchantment.").targets(&[targets::ENCHANTMENT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::DESTROY_EVIL };
