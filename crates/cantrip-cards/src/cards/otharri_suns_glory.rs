use cantrip_api::*;
use cantrip_mechanics::tokens::REBEL;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// You get an experience counter (CR 122.1). Then create a 2/2 red Rebel creature token that's tapped and attacking (CR 508.4)
/// for each experience counter you have: the player's running total, the new counter counted, all at once.
fn rebels(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.give_experience(you, 1);
    let count = g.state().player(you).experience as usize;
    let rebel = g.card_id(&REBEL);
    let tokens = vec![(rebel, you); count];
    g.create_tokens_attacking(&tokens, item.subject)?;
    Ok(())
}

fn untapped_rebel(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.controller(id) == asker.controller && !q.obj(id).tapped && q.chars(id).has_subtype(Subtype::Rebel)
}

/// Return this card from your graveyard to the battlefield tapped.
fn return_tapped(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.obj(item.object).zone.is(ZoneKind::Graveyard) {
        g.put_onto_battlefield(item.object, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever Otharri attacks, you get an experience counter. Then create a 2/2 red Rebel creature token that's tapped and attacking for each experience counter you have.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks_defender,
            effect: rebels,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}{R}{W}, Tap an untapped Rebel you control: Return this card from your graveyard to the battlefield tapped.",
            cost: Cost { mana: mana(2, 1, 0, 0, 1, 0), ..Cost::tap_untapped(1, untapped_rebel) },
            from: ZoneKind::Graveyard,
            effect: return_tapped,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::OTHARRI_SUNS_GLORY
};

pub static EXTRA: &[&CardDef] = &[&REBEL];
