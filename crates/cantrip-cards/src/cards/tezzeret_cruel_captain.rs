use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_hand};
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever an artifact you control enters": Tezzeret himself would count if he were one, so this is not the usual
/// "another permanent" trigger (CR 603.6a).
fn artifact_enters(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, to, .. }
            if to.is(ZoneKind::Battlefield) && q.controller(*new) == q.controller(source) && q.chars(*new).is(CardType::Artifact) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Put a loyalty counter on Tezzeret", if he is still the permanent that triggered.
fn add_loyalty(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Loyalty, 1);
    Ok(())
}

fn artifact_or_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    })
}

/// "Untap target artifact or creature. If it's an artifact creature, put a +1/+1 counter on it."
fn untap_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    g.untap(target);
    let c = g.chars(target);
    if c.is(CardType::Artifact) && c.is(CardType::Creature) {
        g.add_counters(target, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// "Search your library for an artifact card with mana value 1 or less, reveal it, put it into your hand, then
/// shuffle."
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_hand(g, item.controller, item.object, Find::ONE.revealed(), |q, c| {
        q.chars(c).is(CardType::Artifact) && q.mana_value(c) <= 1
    })?;
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

fn artifact_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.controller(id) == asker.controller && q.chars(id).is(CardType::Artifact))
}

/// "Put three +1/+1 counters on target artifact you control. If it's not a creature, it becomes a 0/0 Robot artifact
/// creature": for good, and the counters are put on first, so it ends up 3/3.
fn grow_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(artifact) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(artifact, CounterKind::PlusOne, 3);
    if !g.chars(artifact).is(CardType::Creature) {
        g.add_effect(item.object, artifact, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
        g.add_effect(item.object, artifact, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Robot)), Duration::Indefinite);
        g.add_effect(item.object, artifact, EffectKind::SetPt(0, 0), Duration::Indefinite);
    }
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Tezzeret, Cruel Captain emblem",
    has_cost: false,
    abilities: &[triggered(cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT, grow_artifact, "At the beginning of combat on your turn, put three +1/+1 counters on target artifact you control. If it's not a creature, it becomes a 0/0 Robot artifact creature.")
        .targets(&[target(artifact_you_control)])],
    text: "At the beginning of combat on your turn, put three +1/+1 counters on target artifact you control. If it's not a creature, it becomes a 0/0 Robot artifact creature.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::ZONE_CHANGE, artifact_enters), add_loyalty, "Whenever an artifact you control enters, put a loyalty counter on Tezzeret."),
        ability(loyalty(0), untap_and_grow, "0: Untap target artifact or creature. If it's an artifact creature, put a +1/+1 counter on it.")
            .targets(&[target(artifact_or_creature)]),
        ability(loyalty(-3), tutor, "\u{2212}3: Search your library for an artifact card with mana value 1 or less, reveal it, put it into your hand, then shuffle."),
        ability(loyalty(-7), emblem, "\u{2212}7: You get an emblem with \"At the beginning of combat on your turn, put three +1/+1 counters on target artifact you control. If it's not a creature, it becomes a 0/0 Robot artifact creature.\""),
    ],
    ..oracle::TEZZERET_CRUEL_CAPTAIN
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
