use cantrip_api::*;
use cantrip_mechanics::empower;
use cantrip_oracle as oracle;

/// "{1}, Exile this card from your graveyard: Empower Jace 2." (CR 722, 701.57b.)
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(EXILE_THIS), empower_jace, "{1}, Exile this card from your graveyard: Empower Jace 2.")
        .from(ZoneKind::Graveyard)],
    ..oracle::CAMPUS_CRIER
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
