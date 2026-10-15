use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, targets};
use cantrip_oracle as oracle;

/// Counter target spell unless its controller pays {1} for each artifact you control (CR 118.12): the artifacts are
/// counted as the spell resolves, and the payment is asked even when it is {0} (CR 118.5).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let artifacts = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Artifact)).count();
    counter_unless_pays(g, item, 0, mana(artifacts as u8, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[targets::SPELL], effect }), ..oracle::OVERRIDE };
