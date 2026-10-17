use cantrip_api::*;
use cantrip_mechanics::{Animation, animate};
use cantrip_oracle as oracle;

/// "{1}: This land becomes a 1/1 Phyrexian Blinkmoth artifact creature with flying and infect until end of turn. It's
/// still a land." The Artifact and Creature types and the creature types Phyrexian and Blinkmoth (words of its text,
/// read through its words, CR 612) are added (layer 4, CR 205.1b), flying and infect granted (layer 6) and its base
/// power and toughness set to 1/1 (layer 7b), so counters and pumps count on top, until the cleanup step (CR 514.2).
/// The text names no color, so it stays colorless. Infect is the core's (CR 702.90): its damage is -1/-1 counters to a
/// creature and poison counters to a player. Being a creature only from now, it can attack or tap for mana only if its
/// controller has controlled it continuously since the turn began (CR 302.6).
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let animation = Animation {
        power: 1,
        toughness: 1,
        colors: None,
        types: TypeSet::of(&[CardType::Artifact]),
        subtypes: &[words.subtype(Subtype::Phyrexian), words.subtype(Subtype::Blinkmoth)],
        all_creature_types: false,
        keywords: Keywords::of(&[Keyword::Flying, Keyword::Infect]),
        ability: None,
    };
    animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{1}"), becomes_creature, "{1}: This land becomes a 1/1 Phyrexian Blinkmoth artifact creature with flying and infect until end of turn. It's still a land."),
    ],
    ..oracle::INKMOTH_NEXUS
};
