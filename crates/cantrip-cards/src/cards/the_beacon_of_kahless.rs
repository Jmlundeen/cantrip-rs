use cantrip_api::*;
use cantrip_mechanics::battlefront::opponents_attacked_this_turn;
use cantrip_oracle as oracle;

/// "Battlefront — {T}: Choose a color. Add an amount of mana of that color equal to the number of opponents you attacked
/// this turn." (CR 508.1, 106.5): the color is named as the ability is activated, the opponents counted as it resolves
/// (a mana ability, CR 605.3b); Rite of Ascension's shared count.
fn add_for_attacked_opponents(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    let Some(color) = color else { return Ok(()) };
    let amount = opponents_attacked_this_turn(g, asker.controller);
    if amount == 0 {
        return Ok(());
    }
    g.add_mana(asker.controller, color.mana(), amount as u32);
    Ok(())
}

/// "{T}: Add one mana of any color." (CR 605.1a) and the Battlefront tap above: Great Forest Druid's shared any-color tap
/// beside the new count.
pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        mana_ability(TAP, add_any_color(1).by(add_for_attacked_opponents), "Battlefront — {T}: Choose a color. Add an amount of mana of that color equal to the number of opponents you attacked this turn."),
    ],
    ..oracle::THE_BEACON_OF_KAHLESS
};
