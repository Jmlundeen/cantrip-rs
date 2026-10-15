use cantrip_api::*;
use cantrip_mechanics::{targets, destroy_target};
use cantrip_oracle as oracle;

fn battle_display(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

/// The Adventure (CR 715): destroy target artifact.
pub static BATTLE_DISPLAY: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::ARTIFACT], effect: battle_display }),
    ..oracle::BATTLE_DISPLAY
};

pub static CARD: CardDef = CardDef { adventure: Some(&BATTLE_DISPLAY), ..oracle::EMBERETH_SHIELDBREAKER };

pub static EXTRA: &[&CardDef] = &[&BATTLE_DISPLAY];
