use cantrip_api::*;
use cantrip_mechanics::surveil;
use cantrip_oracle as oracle;

/// "Surveil X, where X is the number of artifacts you control. Then draw three cards." X is counted once, as the spell
/// resolves (the ruling): 0 with no artifact, and then there is no surveil at all and nothing that triggers on one
/// triggers (CR 701.25c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let x = g.permanents_of(you).into_iter().filter(|&p| g.chars(p).is(CardType::Artifact)).count();
    surveil(g, you, item.object, x)?;
    g.draw(you, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::CEREBRAL_DOWNLOAD };
