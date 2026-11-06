use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target permanent. Its controller creates a 3/3 green Beast creature token." (CR 701.8a, 111.2).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::destroy_and_its_controller_creates(g, item, &tokens::BEAST_3_3)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT], effect),
    ..oracle::BEAST_WITHIN
};

pub static EXTRA: &[&CardDef] = &[&tokens::BEAST_3_3];
