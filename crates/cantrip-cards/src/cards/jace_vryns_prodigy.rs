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
    abilities: &[triggered(cantrip_mechanics::triggers::YOU_CAST_A_SPELL, mill_five, "Whenever you cast a spell, target opponent mills five cards.")
        .targets(&[targets::OPPONENT])],
    text: "Whenever you cast a spell, target opponent mills five cards.",
    ..CardDef::BLANK
};

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Up to one target creature gets -2/-0 until your next turn.")
            .targets(&[targets::CREATURE.optional()]),
        ability(loyalty(-3), grant_flashback, "\u{2212}3: You may cast target instant or sorcery card from your graveyard this turn. If that spell would be put into your graveyard, exile it instead.")
            .targets(&[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD]),
        ability(loyalty(-9), emblem, "\u{2212}9: You get an emblem with \"Whenever you cast a spell, target opponent mills five cards.\""),
    ],
    ..oracle::JACE_TELEPATH_UNBOUND
};

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, loot, "{T}: Draw a card, then discard a card. If there are five or more cards in your graveyard, exile Jace, then return him to the battlefield transformed under his owner's control.")],
    back: Some(&BACK),
    ..oracle::JACE_VRYNS_PRODIGY
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
