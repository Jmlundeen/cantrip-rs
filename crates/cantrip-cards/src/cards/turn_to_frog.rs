use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Until end of turn, target creature loses all abilities and becomes a blue Frog with base power and toughness 1/1."
fn frog(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::loses_all_abilities_and_becomes(g, item, 0, Color::Blue, Subtype::Frog, 1, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], frog), ..oracle::TURN_TO_FROG };
