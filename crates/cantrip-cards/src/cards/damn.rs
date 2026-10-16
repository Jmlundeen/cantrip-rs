use cantrip_api::*;
use cantrip_mechanics::{overload, targets};
use cantrip_oracle as oracle;

/// Destroy target creature. A creature destroyed this way can't be regenerated. Overloaded (CR 702.96a), destroy each
/// creature, all at once.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = overload::affected(g, item, |chars, _| chars.is(CardType::Creature));
    g.destroy_all_no_regen(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    overload: Some(mana("{2}{W}{W}")),
    ..oracle::DAMN
};
