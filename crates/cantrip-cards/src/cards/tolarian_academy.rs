use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {U} for each artifact you control." (CR 106.5): counted as the ability resolves, possibly none.
fn add_blue_for_each_artifact(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let artifacts = g
        .permanents_of(asker.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Artifact))
        .count();
    if artifacts > 0 {
        g.add_mana(asker.controller, Mana::U, artifacts as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP, add_by(add_blue_for_each_artifact), "{T}: Add {U} for each artifact you control.")],
    ..oracle::TOLARIAN_ACADEMY
};
