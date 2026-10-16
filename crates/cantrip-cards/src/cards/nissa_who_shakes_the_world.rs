use cantrip_api::*;
use cantrip_mechanics::search_library;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens;
use cantrip_oracle as oracle;

/// "Whenever you tap a Forest for mana" (CR 605.1b): a Forest by its land type, so a dual land with the type taps for the extra
/// {G} too.
fn forest_tapped_for_mana(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::TappedForMana { permanent, player }
            if *player == q.controller(source) && q.chars(*permanent).has_subtype(q.words(source).subtype(Subtype::Forest)) =>
        {
            Fired::about(Target::Object(*permanent))
        }
        _ => None,
    }
}

/// "Add an additional {G}": a triggered mana ability resolves where the tap happens (CR 605.4a).
fn add_green(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::G, 1);
    Ok(())
}

fn noncreature_land_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id)
        && q.controller(id) == asker.controller && { let c = q.chars(id); c.is(CardType::Land) && !c.is(CardType::Creature) })
}

/// +1: Put three +1/+1 counters on up to one target noncreature land you control. Untap it. It becomes a 0/0 Elemental creature
/// with vigilance and haste that's still a land. It states no duration (CR 611.2a), so the land stays a creature: the type added
/// rather than set (CR 205.1b), and the base 0/0 in layer 7b under the counters of 7c.
fn animate_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.target_count == 0 || !g.target_legal(item, 0) {
        return Ok(());
    }
    let Target::Object(land) = item.targets[0] else { return Ok(()) };
    g.add_counters(land, CounterKind::PlusOne, 3);
    g.untap(land);
    g.add_effect(item.object, land, EffectKind::AddTypes(TypeSet::of(&[CardType::Creature])), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Elemental)), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::AddKeywords(Keywords::of(&[Keyword::Vigilance, Keyword::Haste])), Duration::Indefinite);
    g.add_effect(item.object, land, EffectKind::SetPt(0, 0), Duration::Indefinite);
    Ok(())
}

/// -8: You get an emblem with "Lands you control have indestructible." Search your library for any number of Forest cards, put
/// them onto the battlefield tapped, then shuffle.
fn emblem_and_forests(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    let you = item.controller;
    let forests = g.state().zone(Zone::library(you)).len().min(255) as u8;
    let forest = g.words(item.object).subtype(Subtype::Forest);
    let found = search_library(g, you, you, item.object, forests, true, |q, c| q.chars(c).has_subtype(forest))?;
    let lands: Vec<(ObjectId, PlayerId, bool)> = found.into_iter().map(|c| (c, you, true)).collect();
    g.put_all_onto_battlefield_as(&lands)?;
    g.shuffle(you);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

fn land_its_controller_controls(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Land) && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn indestructible(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = Keywords(chars.keywords.0 | Keywords::of(&[Keyword::Indestructible]).0);
}

pub static EMBLEM: CardDef = CardDef {
    name: "Nissa, Who Shakes the World emblem",
    has_cost: false,
    statics: &[StaticDef {
        text: "Lands you control have indestructible.",
        layer: Layer::Ability,
        affects: land_its_controller_controls,
        apply: indestructible,
        ..StaticDef::BLANK
    }],
    text: "Lands you control have indestructible.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when(EventMask::TAPPED, forest_tapped_for_mana), add_green, "Whenever you tap a Forest for mana, add an additional {G}.")
            .mana_ability(),
        ability(loyalty(1), animate_land, "+1: Put three +1/+1 counters on up to one target noncreature land you control. Untap it. It becomes a 0/0 Elemental creature with vigilance and haste that's still a land.")
            .targets(&[TargetSpec { min: 0, max: 1, filter: noncreature_land_you_control }]),
        ability(loyalty(-8), emblem_and_forests, "\u{2212}8: You get an emblem with \"Lands you control have indestructible.\" Search your library for any number of Forest cards, put them onto the battlefield tapped, then shuffle."),
    ],
    ..oracle::NISSA_WHO_SHAKES_THE_WORLD
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
