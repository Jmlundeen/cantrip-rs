use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_mechanics::triggers::{PROWESS, this_enters};
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

fn level(q: &dyn Query, source: ObjectId) -> u8 {
    q.obj(source).class_level.max(1)
}

fn at_level_one(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 1
}

fn at_level_two(q: &dyn Query, asker: &Asker) -> bool {
    level(q, asker.source) == 2
}

fn become_level_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 2);
    Ok(())
}

fn become_level_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.set_class_level(item.object, 3);
    Ok(())
}

fn became_level_two(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ClassLevel { object, level: 2 } if *object == source => Fired::PLAIN,
        _ => None,
    }
}

fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

/// "Whenever you cast an instant or sorcery spell", at level 3.
fn cast_at_level_three(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source)
                && level(q, source) >= 3
                && (q.chars(*spell).is(CardType::Instant) || q.chars(*spell).is(CardType::Sorcery)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{U}: Level 2",
            cost: Cost::mana(mana(3, 0, 1, 0, 0, 0)),
            timing: Timing::Sorcery,
            condition: Some(at_level_one),
            effect: become_level_two,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{5}{U}: Level 3",
            cost: Cost::mana(mana(5, 0, 1, 0, 0, 0)),
            timing: Timing::Sorcery,
            condition: Some(at_level_two),
            effect: become_level_three,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this Class enters, create a 1/1 blue and red Otter creature token with prowess.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: otter,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this Class becomes level 2, return target instant or sorcery card from your graveyard to your hand.",
            listens: EventMask::CLASS_LEVEL,
            matches: became_level_two,
            targets: &[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD],
            effect: return_to_hand,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you cast an instant or sorcery spell, create a 1/1 blue and red Otter creature token with prowess.",
            listens: EventMask::SPELL_CAST,
            matches: cast_at_level_three,
            effect: otter,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::STORMCHASERS_TALENT
};

pub static EXTRA: &[&CardDef] = &[&OTTER];
