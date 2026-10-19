use cantrip_api::*;
use cantrip_mechanics::triggers::A_PLAYER_CASTS_A_NONCREATURE_SPELL;
use cantrip_oracle as oracle;

/// "They lose 2 life" (CR 119.3): "they" is the player who cast the spell, whoever controls Mai, carried as the ability's
/// subject when it triggered (CR 603.2).
fn caster_loses_two_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(player) = item.subject {
        g.lose_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        A_PLAYER_CASTS_A_NONCREATURE_SPELL,
        caster_loses_two_life,
        "Whenever a player casts a noncreature spell, they lose 2 life.",
    )],
    ..oracle::MAI_SCORNFUL_STRIKER
};
