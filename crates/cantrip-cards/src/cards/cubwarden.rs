use cantrip_api::*;
use cantrip_mechanics::{mutate, tokens};
use cantrip_oracle as oracle;

/// "Create two 1/1 white Cat creature tokens with lifelink."
fn make_two_lifelink_cats(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::CAT_LIFELINK, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[mutate::way(mana("{2}{W}{W}"), "Mutate {2}{W}{W}")],
    abilities: &[triggered(mutate::THIS_MUTATES, make_two_lifelink_cats, "Whenever this creature mutates, create two 1/1 white Cat creature tokens with lifelink.")],
    ..oracle::CUBWARDEN
};

pub static EXTRA: &[&CardDef] = &[&tokens::CAT_LIFELINK];
