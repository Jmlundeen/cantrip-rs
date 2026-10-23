use cantrip_api::*;
use cantrip_mechanics::{fight, history, pump, targets};
use cantrip_oracle as oracle;

/// "If you cast this spell during your main phase, the creature you control gets +1/+1 until end of turn. Then those
/// creatures fight each other." Whether it was cast in a main phase is the cast record's fact (CR 601.2i). The pump ends a
/// sentence (CR 608.2c) before the fight; each target is judged on its own (CR 608.2b), so the creature you control is
/// pumped even when the other became illegal, while the fight needs both (CR 701.14b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.history().fact_as_cast(item.object) != 0 {
        if let Some(mine) = targets::object(g, item, 0) {
            pump(g, item.object, mine, 1, 1);
        }
    }
    g.end_sentence();
    fight::targets_fight(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_YOU_DONT_CONTROL], effect),
    cast_fact: Some(history::cast_during_your_main_phase),
    ..oracle::TAIL_SWIPE
};
