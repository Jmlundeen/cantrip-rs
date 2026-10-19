use cantrip_api::*;
use cantrip_mechanics::{empower, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +2/+2 and gains flying until end of turn. Empower Jace 2." An illegal target prevents
/// empower too (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 2, 2);
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])), Duration::EndOfTurn);
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::ACADEMIC_ASCENT
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
