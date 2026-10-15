use cantrip_api::*;
use cantrip_mechanics::triggers::this_dies;
use cantrip_oracle as oracle;

fn add_any(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{1}, {T}, Sacrifice this artifact: Add one mana of any color.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
            mana: Some(add_any),
            any_color: true,
            adds_any: 1,
            ..ActivatedDef::BLANK
        }),
        // Sacrificed for its own mana, it still draws (CR 603.10a).
        AbilityDef::Triggered(TriggeredDef {
            text: "When Chromatic Star is put into a graveyard from the battlefield, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            effect: cantrip_mechanics::effects::draw::<1>,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::CHROMATIC_STAR
};
