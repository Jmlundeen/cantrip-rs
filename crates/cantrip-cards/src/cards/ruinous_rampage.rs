use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Ruinous Rampage deals 3 damage to each opponent." (CR 120.3, 102.2): one damage event for each opponent still in the game,
/// dealt together.
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::damage_each_opponent(g, item, 3);
    Ok(())
}

/// "Exile all artifacts with mana value 3 or less." Every such artifact on the battlefield, whoever controls it, in one move
/// (CR 406.2, 608.2c); an {X} in a mana cost is 0 on the battlefield (CR 202.3e), which `mana_value` reads.
fn exile_cheap_artifacts(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let artifacts: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Artifact) && g.mana_value(id) <= 3).collect();
    g.exile_all(&artifacts);
    Ok(())
}

/// Choose one (CR 700.2).
static MODAL: ModalDef = modes(&[
    mode(damage, "Ruinous Rampage deals 3 damage to each opponent."),
    mode(exile_cheap_artifacts, "Exile all artifacts with mana value 3 or less."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RUINOUS_RAMPAGE };
