use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays_reporting, targets, tokens};
use cantrip_oracle as oracle;

/// "Counter target spell unless its controller pays {2}. If they do, you create a Lander token" (CR 118.12, 701.6, 111.10u): the Lander
/// is made for Divert Disaster's controller only when the spell's controller paid; if the spell was countered, or the target was no
/// longer legal, there is none.
fn counter_unless_they_pay_two_then_lander(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some((_, true)) = counter_unless_pays_reporting(g, item, 0, mana("{2}"))? {
        tokens::create_lander(g, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], counter_unless_they_pay_two_then_lander),
    ..oracle::DIVERT_DISASTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::LANDER];
