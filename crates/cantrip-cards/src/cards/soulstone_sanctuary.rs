use cantrip_api::*;
use cantrip_mechanics::{Animation, animate};
use cantrip_oracle as oracle;

/// "{4}: This land becomes a 3/3 creature with vigilance and all creature types. It's still a land." The Creature type
/// and every creature type are added (layer 4, CR 205.1b, 205.3m), vigilance granted (layer 6) and its base power and
/// toughness set (layer 7b), so counters and pumps count on top. The text names no color, so it stays colorless, and gives
/// no duration (CR 611.2a, the ruling): it lasts until the land leaves the battlefield (CR 400.7) or a later effect
/// changes it. All creature types is not the Changeling ability (the Mutavault ruling).
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let animation = Animation {
        power: 3,
        toughness: 3,
        colors: None,
        types: TypeSet::NONE,
        subtypes: &[],
        all_creature_types: true,
        keywords: Keywords::of(&[Keyword::Vigilance]),
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::Indefinite);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{4}"), becomes_creature, "{4}: This land becomes a 3/3 creature with vigilance and all creature types. It's still a land."),
    ],
    ..oracle::SOULSTONE_SANCTUARY
};
