use cantrip_api::*;
use cantrip_mechanics::effects::destroy_all_creatures;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Destroy all creatures, then any number of target players each mill half their library, rounded down." The creatures go
/// first (CR 701.8a), so what they did while on the battlefield is over before the mill; then each chosen player who is still a
/// legal target (CR 608.2b) mills half of their library as it stands now, rounded down (CR 107.1a), all at once (CR 101.4,
/// 701.17). With no target chosen nobody mills; if every target chosen is illegal, the spell does not resolve at all.
fn destroy_all_then_mill_half(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_all_creatures(g, item)?;
    let mills: Vec<(PlayerId, u32)> =
        targets::legal_players(g, item).into_iter().map(|p| (p, g.state().zone(Zone::library(p)).len() as u32 / 2)).collect();
    g.mill_together(&mills);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::PLAYER.between(0, TargetSpec::ANY_NUMBER).labelled("Players who each mill half their library, rounded down")],
        destroy_all_then_mill_half,
    ),
    ..oracle::SINGULARITY_RUPTURE
};
