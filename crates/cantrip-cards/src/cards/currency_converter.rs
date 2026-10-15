use cantrip_api::*;
use cantrip_mechanics::{choose, discard_chosen, tokens};
use cantrip_oracle as oracle;

/// "Whenever you discard a card": the card, in the graveyard, is what the ability is about.
fn you_discard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { player, card } if *player == q.controller(source) => Fired::about(Target::Object(*card)),
        _ => None,
    }
}

/// "You may exile that card from your graveyard": exiled with this artifact (CR 607.2a), where its other ability
/// finds it. A card that has left the graveyard since is not the card any more (CR 400.7).
fn exile_discarded(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    if !is_live(g, card) || !g.obj(card).zone.is(ZoneKind::Graveyard) {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, item.controller, item.object, Purpose::Optional)? {
        g.exile_with(&[card], item.object, false);
    }
    Ok(())
}

fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static ROGUE: CardDef = CardDef {
    name: "Rogue",
    has_cost: false,
    colors: Some(ColorSet::of(Color::Black)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Rogue]),
    power: Some(2),
    toughness: Some(2),
    ..CardDef::BLANK
};

/// "Put a card exiled with this artifact into its owner's graveyard. If it's a land card, create a Treasure token. If
/// it's a nonland card, create a 2/2 black Rogue creature token." A lone card in the store is no choice; an empty store
/// does nothing at all. Whether it is a land is read while it is still in exile.
fn cash_in(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let store = g.exiled_with(item.object);
    let Some(&first) = store.first() else { return Ok(()) };
    let mut chosen = first;
    if store.len() > 1 {
        let Some(card) = choose::card(g, item.controller, item.object, Purpose::PutIntoGraveyard, store)? else { return Ok(()) };
        chosen = card;
    }
    let land = g.chars(chosen).is(CardType::Land);
    g.put_into_graveyard(&[chosen]);
    if land {
        tokens::create_treasure(g, item.controller)
    } else {
        tokens::create(g, item.controller, &ROGUE, 1)?;
        Ok(())
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you discard a card, you may exile that card from your graveyard.",
            listens: EventMask::DISCARDED,
            matches: you_discard,
            effect: exile_discarded,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}, {T}: Draw a card, then discard a card.",
            cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: loot,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Put a card exiled with this artifact into its owner's graveyard. If it's a land card, create a Treasure token. If it's a nonland card, create a 2/2 black Rogue creature token.",
            cost: Cost { tap: true, ..Cost::NONE },
            effect: cash_in,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::CURRENCY_CONVERTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE, &ROGUE];
