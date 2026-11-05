use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Choose two" (CR 700.2): the two modes are carried out in the order printed (CR 700.2d). The two creature modes read the
/// mana value of each creature as it is now (CR 202.3), through the shared mana-value-bounded destruction.
static MODAL: ModalDef = modes(&[
    mode(effects::destroy_all_of_type::<{ CardType::Artifact as u8 }>, "Destroy all artifacts."),
    mode(effects::destroy_all_of_type::<{ CardType::Enchantment as u8 }>, "Destroy all enchantments."),
    mode(effects::destroy_creatures_with_mana_value_at_most::<3>, "Destroy all creatures with mana value 3 or less."),
    mode(effects::destroy_creatures_with_mana_value_at_least::<4>, "Destroy all creatures with mana value 4 or greater."),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AUSTERE_COMMAND };
