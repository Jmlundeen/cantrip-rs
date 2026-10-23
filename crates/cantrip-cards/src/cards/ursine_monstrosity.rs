use cantrip_api::*;
use cantrip_oracle as oracle;

/// Mill a card and choose an opponent at random. This creature attacks that player this combat if able. Until end of turn, it
/// gains indestructible and gets +1/+1 for each card type among cards in your graveyard (counted as the ability resolves).
fn combat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = item.controller;
    g.mill(me, 1);
    let opponents: Vec<PlayerId> = g.state().opponents(me).collect();
    if !opponents.is_empty() {
        if let Some(chosen) = cantrip_mechanics::library::random_of(g, &opponents) {
            g.add_effect(item.object, item.object, EffectKind::MustAttackPlayer(chosen), Duration::EndOfCombat);
        }
    }
    g.end_sentence();
    let types = g.state().zone(Zone::graveyard(me)).iter().fold(0u16, |all, &card| all | g.chars(card).types.0).count_ones() as i16;
    instruction(g, |g| {
        g.add_effect(item.object, item.object, EffectKind::Pump(types, types), Duration::EndOfTurn);
        cantrip_mechanics::effects::gain_indestructible_until_end_of_turn(g, item.object, item.object);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT, combat, "At the beginning of combat on your turn, mill a card and choose an opponent at random. This creature attacks that player this combat if able. Until end of turn, this creature gains indestructible and gets +1/+1 for each card type among cards in your graveyard.")],
    ..oracle::URSINE_MONSTROSITY
};
