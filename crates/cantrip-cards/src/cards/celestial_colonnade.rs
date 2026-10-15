use cantrip_api::*;
use cantrip_mechanics::{Animation, animate, mana};
use cantrip_oracle as oracle;

fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 4,
        toughness: 4,
        colors: Some(words.colors(ColorSet(0b00011))),
        subtype: Some(words.subtype(Subtype::Elemental)),
        keywords: Keywords::of(&[Keyword::Flying, Keyword::Vigilance]),
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana::tap_for_either(Color::White, Color::Blue, "{T}: Add {W} or {U}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{W}{U}: Until end of turn, this land becomes a 4/4 white and blue Elemental creature with flying and vigilance. It's still a land.",
            cost: Cost { mana: mana(3, 1, 1, 0, 0, 0), ..Cost::NONE },
            effect: becomes_creature,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CELESTIAL_COLONNADE
};
