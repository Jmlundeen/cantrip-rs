use cantrip_api::*;
use cantrip_mechanics::mana::{add_c, tap_for};
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_mechanics::tokens::{self, CONSTRUCT};
use cantrip_oracle as oracle;

/// I: "This Saga gains '{T}: Add {C}.'", for as long as it stays (CR 611.2a).
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.obj(item.object).card;
    g.add_effect(item.object, item.object, EffectKind::AddAbility(AbilityRef::delayed(me, 0)), Duration::Indefinite);
    Ok(())
}

/// II: "This Saga gains '{2}, {T}: Create a 0/0 colorless Construct ...'", independently of its lore counters.
fn chapter_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.obj(item.object).card;
    g.add_effect(item.object, item.object, EffectKind::AddAbility(AbilityRef::delayed(me, 1)), Duration::Indefinite);
    Ok(())
}

fn make_construct(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CONSTRUCT, 1)?;
    Ok(())
}

/// An artifact card whose mana cost is exactly {0} or {1}: no colored, colorless or hybrid symbol, no X.
fn cheap_artifact(q: &dyn Query, id: ObjectId) -> bool {
    let card = q.card(q.obj(id).card);
    let c = &card.cost;
    card.types.has(CardType::Artifact)
        && card.has_cost
        && c.generic <= 1
        && c.symbols == [0; 6]
        && c.hybrid == [None; 6]
        && c.phyrexian == [0; 5]
        && c.x == 0
}

/// III: search for an artifact card with mana cost {0} or {1}, put it onto the battlefield, then shuffle.
fn chapter_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, cheap_artifact)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "I \u{2014} This Saga gains \"{T}: Add {C}.\"",
            chapter: 1,
            effect: chapter_one,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "II \u{2014} This Saga gains \"{2}, {T}: Create a 0/0 colorless Construct artifact creature token with 'This token gets +1/+1 for each artifact you control.'\"",
            chapter: 2,
            effect: chapter_two,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "III \u{2014} Search your library for an artifact card with mana cost {0} or {1}, put it onto the battlefield, then shuffle.",
            chapter: 3,
            effect: chapter_three,
            ..TriggeredDef::BLANK
        }),
    ],
    delayed: &[
        tap_for(add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}, {T}: Create a 0/0 colorless Construct artifact creature token with \"This token gets +1/+1 for each artifact you control.\"",
            cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: make_construct,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::URZAS_SAGA
};

pub static EXTRA: &[&CardDef] = &[&CONSTRUCT];
