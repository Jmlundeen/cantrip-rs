use cantrip_api::*;
use cantrip_mechanics::{Animation, animate};
use cantrip_oracle as oracle;

/// "{1}: This land becomes a 2/2 creature with all creature types until end of turn. It's still a land." The Creature
/// type and every creature type are added (layer 4, CR 205.1b, 205.3m) and its base power and toughness set (layer 7b)
/// until the cleanup step (CR 514.2). The text names no color, so it stays colorless. All creature types is not the
/// Changeling ability (the card's ruling).
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let animation = Animation {
        power: 2,
        toughness: 2,
        colors: None,
        types: TypeSet::NONE,
        subtypes: &[],
        all_creature_types: true,
        keywords: Keywords::NONE,
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{1}"), becomes_creature, "{1}: This land becomes a 2/2 creature with all creature types until end of turn. It's still a land."),
    ],
    ..oracle::MUTAVAULT
};
