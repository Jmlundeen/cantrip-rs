use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}: Add {G} for each creature you control." (CR 106.5): counted as the ability resolves, possibly none.
fn add_green_for_each_creature(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let creatures = g
        .permanents_of(asker.controller)
        .into_iter()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .count();
    if creatures > 0 {
        g.add_mana(asker.controller, Mana::G, creatures as u8);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}: Add {G} for each creature you control.",
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_green_for_each_creature),
        ..ActivatedDef::BLANK
    })],
    ..oracle::GAEAS_CRADLE
};
