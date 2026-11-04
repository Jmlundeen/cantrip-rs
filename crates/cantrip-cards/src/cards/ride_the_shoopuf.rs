use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_mechanics::{Animation, animate, effects, targets};
use cantrip_oracle as oracle;

/// "{5}{G}{G}: This enchantment becomes a 7/7 Beast creature in addition to its other types." The Creature and Beast types
/// are added (layer 4, CR 205.1b) and the base power and toughness set (layer 7b), so counters count on top. The text names
/// no color and gives no duration (CR 611.2a, the ruling): it lasts until the enchantment leaves the battlefield (CR 400.7)
/// or a later effect changes it. The Beast type is read through the words of this card's text (CR 612).
fn becomes_a_beast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let animation = Animation {
        power: 7,
        toughness: 7,
        colors: None,
        types: TypeSet::NONE,
        subtypes: &[g.words(item.object).subtype(Subtype::Beast)],
        all_creature_types: false,
        replaces_creature_types: false,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            LANDFALL,
            effects::plus_one_counter_on_target,
            "Landfall \u{2014} Whenever a land you control enters, put a +1/+1 counter on target creature you control.",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL]),
        ability(cost("{5}{G}{G}"), becomes_a_beast, "{5}{G}{G}: This enchantment becomes a 7/7 Beast creature in addition to its other types."),
    ],
    ..oracle::RIDE_THE_SHOOPUF
};
