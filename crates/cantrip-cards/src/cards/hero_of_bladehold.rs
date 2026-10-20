use cantrip_api::*;
use cantrip_mechanics::battle_cry;
use cantrip_mechanics::combat::create_attacking_of_your_choice;
use cantrip_mechanics::tokens::SOLDIER;
use cantrip_oracle as oracle;

/// Two 1/1 white Soldier tokens, tapped and attacking (CR 508.4): their controller chooses what each one attacks as it
/// enters.
fn soldiers(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let soldier = g.card_id(&SOLDIER);
    create_attacking_of_your_choice(g, item.controller, item.object, soldier, 2, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        battle_cry::TRIGGER,
        triggered(cantrip_mechanics::triggers::THIS_ATTACKS, soldiers, "Whenever Hero of Bladehold attacks, create two 1/1 white Soldier creature tokens that are tapped and attacking."),
    ],
    ..oracle::HERO_OF_BLADEHOLD
};

pub static EXTRA: &[&CardDef] = &[&SOLDIER];
