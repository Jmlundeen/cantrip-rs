use cantrip_api::*;
use cantrip_mechanics::{affinity, effects::gain_indestructible_until_end_of_turn, tokens, triggers};
use cantrip_oracle as oracle;

/// The 3/4 green Treefolk creature token with reach that Sapling Nursery makes.
pub static TREEFOLK: CardDef = CardDef {
    name: "Treefolk",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Green)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Treefolk]),
    power: Some(3),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Reach]),
    ..CardDef::BLANK
};

/// "Create a 3/4 green Treefolk creature token with reach" (CR 111.1): one for each land that enters, since each is its own
/// trigger (CR 603.2).
fn create_treefolk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &TREEFOLK, 1)?;
    Ok(())
}

/// "Treefolk and Forests you control gain indestructible until end of turn" (CR 611.2c): the permanents of the ability's
/// controller that are Treefolk or Forests as it resolves, and no permanent that comes later. The cost exiled the enchantment
/// (CR 602.2b), which is the effect's source as it last was (CR 113.7a). Treefolk and Forest are words of this card's rules
/// text (CR 612), and a permanent is one when the layers made it that type.
fn protect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let words = g.words(item.object);
    let (treefolk, forest) = (words.subtype(Subtype::Treefolk), words.subtype(Subtype::Forest));
    for permanent in g.permanents_of(item.controller) {
        let subtypes = g.chars(permanent).subtypes;
        if subtypes.has(treefolk) || subtypes.has(forest) {
            gain_indestructible_until_end_of_turn(g, item.object, permanent);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_land_type::<{ Subtype::Forest as usize }>],
    abilities: &[
        triggered(when_asked(EventMask::ZONE_CHANGE, triggers::landfall), create_treefolk, "Landfall \u{2014} Whenever a land you control enters, create a 3/4 green Treefolk creature token with reach."),
        ability(cost("{1}{G}").and(EXILE_THIS), protect, "{1}{G}, Exile this enchantment: Treefolk and Forests you control gain indestructible until end of turn."),
    ],
    ..oracle::SAPLING_NURSERY
};

pub static EXTRA: &[&CardDef] = &[&TREEFOLK];
