use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// {T}: Add {G}{G}. If you've completed a dungeon, add six {G} instead (CR 309.7, 605.1a): one mana ability, whose
/// yield is read as it is activated.
fn add_green(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let completed = g.state().player(asker.controller).dungeons_completed > 0;
    g.add_mana(asker.controller, Mana::G, if completed { 6 } else { 2 });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Add {G}{G}. If you've completed a dungeon, add six {G} instead.",
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_green),
            adds: ManaPool([0, 0, 0, 0, 2, 0]),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you take the initiative.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::take_the_initiative,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::UNDERMOUNTAIN_ADVENTURER
};
