use cantrip_api::*;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// "You gain 2 life. Create a Lander token." (CR 119.3, 111.10u): the life first, then the token, in the order written (CR 608.2c).
fn gain_life_and_make_a_lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 2)?;
    tokens::create_lander(g, item.controller)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], gain_life_and_make_a_lander), ..oracle::SAMIS_CURIOSITY };

pub static EXTRA: &[&CardDef] = &[&tokens::LANDER];
