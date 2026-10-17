use cantrip_api::*;
use cantrip_mechanics::tokens::create_food;
use cantrip_mechanics::{Animation, animate, mana, targets, triggers};
use cantrip_oracle as oracle;

/// "{2}{B}{G}: This land becomes a 4/4 black and green Horror creature until end of turn. It's still a land."
/// (CR 613.1d, 613.4b, 613.1e)
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 4,
        toughness: 4,
        colors: Some(words.colors(ColorSet(ColorSet::of(Color::Black).0 | ColorSet::of(Color::Green).0))),
        types: TypeSet::NONE,
        subtypes: &[words.subtype(Subtype::Horror)],
        all_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

/// "Whenever this land attacks, create a Food token and exile up to one target card from a graveyard." The Food is
/// made (CR 111.10b) whether or not a target was chosen; an ability whose only target is illegal does not resolve at all (CR 608.2b).
fn food_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_food(g, item.controller)?;
    if item.target_count > 0 {
        cantrip_mechanics::exile_target(g, item, 0);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Black, Color::Green]), "{T}: Add {B} or {G}."),
        ability(cost("{2}{B}{G}"), becomes_creature, "{2}{B}{G}: This land becomes a 4/4 black and green Horror creature until end of turn. It's still a land."),
        triggered(triggers::THIS_ATTACKS, food_and_exile, "Whenever this land attacks, create a Food token and exile up to one target card from a graveyard.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()]),
    ],
    ..oracle::RESTLESS_COTTAGE
};
