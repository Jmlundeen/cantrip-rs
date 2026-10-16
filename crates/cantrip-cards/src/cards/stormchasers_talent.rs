use cantrip_api::*;
use cantrip_mechanics::{class, targets, tokens};
use cantrip_mechanics::triggers::{PROWESS, THIS_ENTERS};
use cantrip_oracle as oracle;

/// The 1/1 blue and red Otter creature token with prowess.
pub static OTTER: CardDef = CardDef {
    name: "Otter",
    has_cost: false,
    colors: Some(ColorSet(ColorSet::of(Color::Blue).0 | ColorSet::of(Color::Red).0)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Otter]),
    power: Some(1),
    toughness: Some(1),
    abilities: &[PROWESS],
    text: "Prowess (Whenever you cast a noncreature spell, this creature gets +1/+1 until end of turn.)",
    ..CardDef::BLANK
};

fn otter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &OTTER, 1)?;
    Ok(())
}

/// "Whenever you cast an instant or sorcery spell", at level 3.
fn cast_at_level_three(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source)
                && class::level(q, source) >= 3
                && (q.chars(*spell).is(CardType::Instant) || q.chars(*spell).is(CardType::Sorcery)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        class::level_two("{3}{U}", "{3}{U}: Level 2"),
        class::level_three("{5}{U}", "{5}{U}: Level 3"),
        triggered(THIS_ENTERS, otter, "When this Class enters, create a 1/1 blue and red Otter creature token with prowess."),
        triggered(class::BECOMES_LEVEL_TWO, cantrip_mechanics::effects::return_first_target_to_hand, "When this Class becomes level 2, return target instant or sorcery card from your graveyard to your hand.")
            .targets(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD]),
        triggered(when(EventMask::SPELL_CAST, cast_at_level_three), otter, "Whenever you cast an instant or sorcery spell, create a 1/1 blue and red Otter creature token with prowess."),
    ],
    ..oracle::STORMCHASERS_TALENT
};

pub static EXTRA: &[&CardDef] = &[&OTTER];
