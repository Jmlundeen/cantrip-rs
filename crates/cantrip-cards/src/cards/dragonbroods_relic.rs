use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets, tokens, TAP_AN_UNTAPPED_CREATURE};
use cantrip_oracle as oracle;

/// The 4/4 Dragon creature token named Reliquary Dragon that is all colors (CR 111.4, 105.2), with flying, lifelink and "When this
/// token enters, it deals 3 damage to any target."
pub static RELIQUARY_DRAGON: CardDef = CardDef {
    name: "Reliquary Dragon",
    has_cost: false,
    colors: Some(ColorSet(
        ColorSet::of(Color::White).0 | ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Black).0 | ColorSet::of(Color::Red).0 | ColorSet::of(Color::Green).0,
    )),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Dragon]),
    power: Some(4),
    toughness: Some(4),
    keywords: Keywords::of(&[Keyword::Flying, Keyword::Lifelink]),
    abilities: &[triggered(THIS_ENTERS, effects::damage_first_target::<3>, "When this token enters, it deals 3 damage to any target.").targets(&[targets::ANY])],
    text: "Flying, lifelink\nWhen this token enters, it deals 3 damage to any target.",
    ..CardDef::BLANK
};

fn make_reliquary_dragon(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &RELIQUARY_DRAGON, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP.and(TAP_AN_UNTAPPED_CREATURE), add_any_color(1), "{T}, Tap an untapped creature you control: Add one mana of any color."),
        ability(
            cost("{3}{W}{U}{B}{R}{G}").and(SACRIFICE_THIS),
            make_reliquary_dragon,
            "{3}{W}{U}{B}{R}{G}, Sacrifice this artifact: Create a 4/4 Dragon creature token named Reliquary Dragon that's all colors. It has flying, lifelink, and \"When this token enters, it deals 3 damage to any target.\" Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::DRAGONBROODS_RELIC
};

pub static EXTRA: &[&CardDef] = &[&RELIQUARY_DRAGON];
