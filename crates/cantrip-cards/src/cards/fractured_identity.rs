use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Exile target nonland permanent. Each player other than its controller creates a token that's a copy of it: its
/// controller and copiable values are read while it is still on the battlefield (CR 608.2h, 707.2).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(target);
    let former = g.controller(target);
    g.exile(target);
    let players: Vec<PlayerId> = g.state().seats().filter(|&p| p != former).collect();
    for player in players {
        g.create_token_copies_of(values, player, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::NONLAND_PERMANENT], effect }),
    ..oracle::FRACTURED_IDENTITY
};
