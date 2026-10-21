use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// {T}: Add {G}{G}. If you've completed a dungeon, add six {G} instead (CR 309.7, 605.1a): one mana ability, whose
/// yield is read as it is activated.
fn add_green(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let completed = cantrip_mechanics::history::completed_a_dungeon(g, asker.controller);
    g.add_mana(asker.controller, Mana::G, if completed { 6 } else { 2 });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::G, 2).by(add_green), "{T}: Add {G}{G}. If you've completed a dungeon, add six {G} instead."),
        triggered(THIS_ENTERS, effects::take_the_initiative, "When this creature enters, you take the initiative."),
    ],
    ..oracle::UNDERMOUNTAIN_ADVENTURER
};
