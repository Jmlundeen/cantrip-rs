use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets, effects, tokens};
use cantrip_oracle as oracle;

/// +1: "Exile the top card of your library. You may cast that card. If you don't, Chandra deals 2 damage to each
/// opponent." The card is offered only if it could be cast now (CR 608.2g): it is not a land and has a legal target
/// for everything it needs. An empty library exiles nothing, and then nothing was cast.
fn impulse(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut cast = false;
    if let Some(&top) = g.state().zone(Zone::library(you)).first() {
        let exiled = g.exile(top);
        if g.obj(exiled).zone.is(ZoneKind::Exile)
            && g.could_cast_during_resolution(exiled)
            && effects::ask_may(g, you, item.object, Purpose::Cast)?
        {
            cast = g.cast_during_resolution(exiled, None)?;
        }
    }
    if !cast {
        let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(you).map(|p| (item.object, Target::Player(p), 2)).collect();
        g.damage_together(&hits);
    }
    Ok(())
}

fn add_red_red(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::R, 2);
    Ok(())
}

fn four_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

fn you_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn five_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 5);
    Ok(())
}

pub static EMBLEM: CardDef = CardDef {
    name: "Chandra, Torch of Defiance emblem",
    has_cost: false,
    abilities: &[triggered(when(EventMask::SPELL_CAST, you_cast), five_damage, "Whenever you cast a spell, this emblem deals 5 damage to any target.")
        .targets(&[targets::ANY])],
    text: "Whenever you cast a spell, this emblem deals 5 damage to any target.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), impulse, "+1: Exile the top card of your library. You may cast that card. If you don't, Chandra deals 2 damage to each opponent."),
        ability(loyalty(1), add_red_red, "+1: Add {R}{R}."),
        ability(loyalty(-3), four_damage, "\u{2212}3: Chandra deals 4 damage to target creature.")
            .targets(&[targets::CREATURE]),
        ability(loyalty(-7), emblem, "\u{2212}7: You get an emblem with \"Whenever you cast a spell, this emblem deals 5 damage to any target.\""),
    ],
    ..oracle::CHANDRA_TORCH_OF_DEFIANCE
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
