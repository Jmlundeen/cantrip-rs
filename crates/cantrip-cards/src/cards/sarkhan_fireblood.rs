use cantrip_api::*;
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

/// +1: "You may discard a card. If you do, draw a card." (`effects::may_discard_to_draw`.)
fn rummage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_discard_to_draw(g, item, 1)
}

/// +1: "Add two mana in any combination of colors. Spend this mana only to cast Dragon spells." A loyalty ability, so no
/// mana ability (its ruling, CR 605.1a): it resolves from the stack, at sorcery speed, and the two colors are chosen then,
/// repeats allowed (CR 106.1b). The mana pays only for casting a spell with the subtype Dragon (CR 106.6, 205.3), the
/// word read through Sarkhan's words (CR 612), as it last was if Sarkhan has gone.
fn dragon_mana(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let dragon = g.words(item.object).subtype(Subtype::Dragon);
    for color in cantrip_mechanics::choose_colors(g, item.controller, item.object, ColorSet(0b11111), 2)? {
        g.add_restricted_mana(item.controller, color.mana(), 1, ManaRestriction::Subtypes(SubtypeSpells::of(dragon)));
    }
    Ok(())
}

/// The 5/5 red Dragon creature token with flying.
pub static DRAGON: CardDef = CardDef {
    name: "Dragon",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Red)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Dragon]),
    power: Some(5),
    toughness: Some(5),
    keywords: Keywords::of(&[Keyword::Flying]),
    text: "Flying",
    ..CardDef::BLANK
};

/// -7: "Create four 5/5 red Dragon creature tokens with flying."
fn dragons(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &DRAGON, 4)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), rummage, "+1: You may discard a card. If you do, draw a card."),
        ability(loyalty(1), dragon_mana, "+1: Add two mana in any combination of colors. Spend this mana only to cast Dragon spells."),
        ability(loyalty(-7), dragons, "\u{2212}7: Create four 5/5 red Dragon creature tokens with flying."),
    ],
    ..oracle::SARKHAN_FIREBLOOD
};

pub static EXTRA: &[&CardDef] = &[&DRAGON];
