use cantrip_api::*;
use cantrip_mechanics::{destroy_target, mana, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Destroy it, if it is still a legal target.
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add(Mana::W), "{T}: Add {W}.")],
    enters: Some(mana::enters_tapped_unless_pay_three_life),
    ..oracle::WITCH_BLESSED_MEADOW
};

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, destroy, "When this creature enters, destroy target artifact or enchantment an opponent controls.")
        .targets(&[targets::ARTIFACT_OR_ENCHANTMENT_AN_OPPONENT_CONTROLS])],
    back: Some(&BACK),
    ..oracle::WITCH_ENCHANTER
};
