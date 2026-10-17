use cantrip_api::*;
use cantrip_mechanics::{effects, empower, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "When this creature enters, empower Jace 3." (CR 701.71a)
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 3)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, empower_jace, "When this creature enters, empower Jace 3."),
        ability(cost("{U}").and(SACRIFICE_THIS), effects::next_spell_cant_be_countered, "{U}, Sacrifice this creature: The next spell you cast this turn can't be countered."),
    ],
    ..oracle::THEORISTS_PROXY
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
