use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Firebending Lesson deals 2 damage to target creature. If this spell was kicked, it deals 5 damage to that creature
/// instead." (CR 702.33: a copy of a kicked spell is kicked, CR 707.10.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, if item.kicked > 0 { 5 } else { 2 });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: kicker(mana("{4}")),
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::FIREBENDING_LESSON
};
