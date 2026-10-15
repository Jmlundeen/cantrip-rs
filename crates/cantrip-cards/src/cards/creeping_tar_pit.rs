use cantrip_api::*;
use cantrip_mechanics::{Animation, animate, mana};
use cantrip_oracle as oracle;

/// "Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be
/// blocked this turn."
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 3,
        toughness: 2,
        colors: Some(words.colors(ColorSet(0b00110))),
        subtype: Some(words.subtype(Subtype::Elemental)),
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    g.add_effect(item.object, item.object, EffectKind::AddKeywords(Keywords::of(&[Keyword::CantBeBlocked])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana::tap_for_either(Color::Blue, Color::Black, "{T}: Add {U} or {B}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{U}{B}: Until end of turn, this land becomes a 3/2 blue and black Elemental creature. It's still a land. It can't be blocked this turn.",
            cost: Cost { mana: mana(1, 0, 1, 1, 0, 0), ..Cost::NONE },
            effect: becomes_creature,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CREEPING_TAR_PIT
};
