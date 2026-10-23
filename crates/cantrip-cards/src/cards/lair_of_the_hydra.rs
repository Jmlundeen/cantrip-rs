use cantrip_api::*;
use cantrip_mechanics::{Animation, animate, mana};
use cantrip_oracle as oracle;

/// "Until end of turn, this land becomes an X/X green Hydra creature. It's still a land." (CR 613.1d, 613.1f, 613.4b): X
/// is the one announced for the ability (CR 107.3a, never 0 here); the color and the creature type are the ones its own
/// text has now (CR 612).
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let x = i16::try_from(item.x).unwrap_or(i16::MAX);
    let animation = Animation {
        power: x,
        toughness: x,
        colors: Some(words.colors(ColorSet::of(Color::Green))),
        types: TypeSet::NONE,
        subtypes: &[words.subtype(Subtype::Hydra)],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_if_two_or_more_other_lands),
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        ability(cost("{X}{G}"), becomes_creature, "{X}{G}: Until end of turn, this land becomes an X/X green Hydra creature. It's still a land. X can't be 0.")
            .x_cant_be_zero(),
    ],
    ..oracle::LAIR_OF_THE_HYDRA
};
