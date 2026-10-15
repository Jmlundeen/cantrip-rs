use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, exile_and_return_transformed, grant_flashback, targets};
use cantrip_oracle as oracle;

/// "{T}: Draw a card, then discard a card. If there are five or more cards in your graveyard, exile Jace, then return him
/// to the battlefield transformed under his owner's control."
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    discard_chosen(g, item.controller, item.object, 1)?;
    if g.state().zone(Zone::graveyard(item.controller)).len() >= 5 {
        let owner = g.obj(item.object).owner;
        exile_and_return_transformed(g, item.object, owner)?;
    }
    Ok(())
}

// --- Jace, Telepath Unbound ---------------------------------------------------------------------------------------

/// "+1: Up to one target creature gets -2/-0 until your next turn."
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0)
        && g.is_on_battlefield(creature)
    {
        g.add_effect(item.object, creature, EffectKind::Pump(-2, 0), Duration::UntilNextTurnOf(item.controller));
    }
    Ok(())
}

/// "Whenever you cast a spell" (CR 603.2): the emblem's owner casts it.
fn you_cast_a_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn mill_five(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Some(player) = item.targets[0].player()
    {
        g.mill(player, 5);
    }
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Jace, Telepath Unbound emblem",
    has_cost: false,
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever you cast a spell, target opponent mills five cards.",
        listens: EventMask::SPELL_CAST,
        matches: you_cast_a_spell,
        targets: &[targets::OPPONENT],
        effect: mill_five,
        ..TriggeredDef::BLANK
    })],
    text: "Whenever you cast a spell, target opponent mills five cards.",
    ..CardDef::BLANK
};

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Up to one target creature gets -2/-0 until your next turn.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[TargetSpec { min: 0, ..targets::CREATURE }],
            effect: plus_one,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}3: You may cast target instant or sorcery card from your graveyard this turn. If that spell would be put into your graveyard, exile it instead.",
            cost: Cost { loyalty: Some(-3), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD],
            effect: grant_flashback,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}9: You get an emblem with \"Whenever you cast a spell, target opponent mills five cards.\"",
            cost: Cost { loyalty: Some(-9), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: emblem,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::JACE_TELEPATH_UNBOUND
};

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}: Draw a card, then discard a card. If there are five or more cards in your graveyard, exile Jace, then return him to the battlefield transformed under his owner's control.",
        cost: Cost { tap: true, ..Cost::NONE },
        effect: loot,
        ..ActivatedDef::BLANK
    })],
    back: Some(&BACK),
    ..oracle::JACE_VRYNS_PRODIGY
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
