use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::{triggers::this_enters, tokens};
use cantrip_oracle as oracle;

/// The 1/1 colorless Hero creature token job select makes.
static HERO: CardDef = CardDef {
    name: "Hero",
    has_cost: false,
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Hero]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// Job select: "create a 1/1 colorless Hero creature token, then attach this to it." An Equipment that has left the
/// battlefield attaches to nothing, and a token a replacement stopped is not there to be equipped (CR 701.3b).
fn job_select(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &HERO, 1)?;
    if let Some(&token) = made.first()
        && is_live(g, token)
        && g.is_on_battlefield(token)
        && is_live(g, item.object)
        && g.is_on_battlefield(item.object)
    {
        g.attach(item.object, token);
    }
    Ok(())
}

/// "Equipped creature is a Wizard in addition to its other types" (layer 4).
fn wizard(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(Subtype::Wizard);
}

/// The equipped creature has the Equipment's delayed ability 0 (CR 613.1f): one triggered ability with two conditions, an
/// ability it controls and that is gone the moment the Equipment stops being attached to it.
fn grant_growth(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

/// "Whenever you cast a noncreature spell and whenever you draw your third card each turn" (CR 603.2, 121.2): the
/// history as it was just before the draw holds the two earlier ones.
fn cast_noncreature_or_draw_third(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { .. } => cantrip_mechanics::triggers::noncreature_spell_you_cast(q, source, event),
        Event::Drew { player, .. } if *player == q.controller(source) && q.history().drawn(*player) == 2 => Fired::PLAIN,
        _ => None,
    }
}

fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Job select (When this Equipment enters, create a 1/1 colorless Hero creature token, then attach this to it.)",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: job_select,
            ..TriggeredDef::BLANK
        }),
        equip(mana(2, 0, 0, 0, 0, 0), "Diana \u{2014} Equip {2}"),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast a noncreature spell and whenever you draw your third card each turn, put a +1/+1 counter on this creature.",
        listens: EventMask(EventMask::SPELL_CAST.0 | EventMask::DREW.0),
        matches: cast_noncreature_or_draw_third,
        effect: grow,
        ..TriggeredDef::BLANK
    })],
    statics: &[
        StaticDef {
            text: "Equipped creature is a Wizard in addition to its other types.",
            layer: Layer::Type,
            affects: attached_object,
            apply: wizard,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has \"Whenever you cast a noncreature spell and whenever you draw your third card each turn, put a +1/+1 counter on this creature.\"",
            layer: Layer::Ability,
            affects: attached_object,
            apply: grant_growth,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::ASTROLOGIANS_PLANISPHERE
};

pub static EXTRA: &[&CardDef] = &[&HERO];
