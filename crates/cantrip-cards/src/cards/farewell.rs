use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Exile all artifacts." Every artifact on the battlefield, whoever controls it, in one move (CR 406.2, 608.2c).
fn exile_artifacts(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let artifacts: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Artifact)).collect();
    g.exile_all(&artifacts);
    Ok(())
}

/// "Exile all creatures."
fn exile_creatures(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    effects::exile_all_creatures(g);
    Ok(())
}

/// "Exile all enchantments."
fn exile_enchantments(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let enchantments: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Enchantment)).collect();
    g.exile_all(&enchantments);
    Ok(())
}

/// "Exile all graveyards."
fn exile_graveyards(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    effects::exile_all_graveyards(g);
    Ok(())
}

/// "Choose one or more" (CR 700.2): the chosen modes are performed in the order written (CR 700.2a).
static MODAL: ModalDef = modes(&[
    mode(exile_artifacts, "Exile all artifacts."),
    mode(exile_creatures, "Exile all creatures."),
    mode(exile_enchantments, "Exile all enchantments."),
    mode(exile_graveyards, "Exile all graveyards."),
])
.choose(1, 4);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::FAREWELL };
