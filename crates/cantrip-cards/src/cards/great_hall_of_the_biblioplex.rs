use cantrip_api::*;
use cantrip_mechanics::mana::{add_c, tap_for};
use cantrip_mechanics::{Animation, animate, pump};
use cantrip_oracle as oracle;

/// "{T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell." (CR 106.6,
/// 605.1a, 119.4)
fn add_restricted(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    if let Some(c) = color {
        g.add_restricted_mana(asker.controller, c.mana(), 1, ManaRestriction::InstantOrSorcerySpells);
    }
    Ok(())
}

/// "{5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with '...' It's still a land." The check is
/// made as the ability resolves; the change has no duration (CR 611.2a), so it lasts while the land stays where it is.
/// The text names no colors, so they are not changed.
fn becomes_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = item.object;
    if g.chars(me).is(CardType::Creature) {
        return Ok(());
    }
    let card = g.obj(me).card;
    let animation = Animation {
        power: 2,
        toughness: 4,
        colors: None,
        subtype: Some(g.words(me).subtype(Subtype::Wizard)),
        keywords: Keywords::NONE,
        ability: Some(AbilityRef::delayed(card, 0)),
    };
    animate(g, me, me, &animation, Duration::Indefinite);
    Ok(())
}

/// "Whenever you cast an instant or sorcery spell" (CR 601.2i): cast, not copied.
fn instant_or_sorcery_you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. }
            if *player == q.controller(source) && (q.chars(*spell).is(CardType::Instant) || q.chars(*spell).is(CardType::Sorcery)) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "This creature gets +1/+0 until end of turn."
fn pump_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        tap_for(add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Pay 1 life: Add one mana of any color. Spend this mana only to cast an instant or sorcery spell.",
            cost: Cost { tap: true, life: 1, ..Cost::NONE },
            mana: Some(add_restricted),
            any_color: true,
            adds_any: 1,
            restricted: Some(ManaRestriction::InstantOrSorcerySpells),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{5}: If this land isn't a creature, it becomes a 2/4 Wizard creature with \"Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn.\" It's still a land.",
            cost: Cost { mana: mana(5, 0, 0, 0, 0, 0), ..Cost::NONE },
            effect: becomes_creature,
            ..ActivatedDef::BLANK
        }),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast an instant or sorcery spell, this creature gets +1/+0 until end of turn.",
        listens: EventMask::SPELL_CAST,
        matches: instant_or_sorcery_you_cast,
        effect: pump_one,
        ..TriggeredDef::BLANK
    })],
    ..oracle::GREAT_HALL_OF_THE_BIBLIOPLEX
};
