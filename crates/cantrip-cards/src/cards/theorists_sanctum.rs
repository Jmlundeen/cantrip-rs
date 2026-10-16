use cantrip_api::*;
use cantrip_mechanics::{behold, empower};
use cantrip_oracle as oracle;

/// "As this land enters, you may behold a Jace. If you don't, this land enters tapped." (CR 701.4a, 614.12a)
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    if !behold::as_enters(g, entering, behold::jace, "Jace")? {
        entering.tapped = true;
    }
    Ok(())
}

/// "{2}{U}, {T}: Empower Jace 2."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 2)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    abilities: &[ability(cost("{2}{U}, {T}"), empower_jace, "{2}{U}, {T}: Empower Jace 2.")],
    ..oracle::THEORISTS_SANCTUM
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
