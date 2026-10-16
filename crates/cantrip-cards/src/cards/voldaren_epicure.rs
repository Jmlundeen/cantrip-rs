use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// It deals 1 damage to each opponent, then creates a Blood token.
fn ping_and_blood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_opponent(g, item, 1);
    tokens::create_blood(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, ping_and_blood, "When Voldaren Epicure enters, it deals 1 damage to each opponent. Create a Blood token.")],
    ..oracle::VOLDAREN_EPICURE
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
