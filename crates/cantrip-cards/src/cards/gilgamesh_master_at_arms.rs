use cantrip_api::*;
use cantrip_mechanics::{choose, library, select, triggers};
use cantrip_oracle as oracle;

/// "Equipment cards" (CR 205.3g, 301.5): a card with the artifact subtype Equipment, the word read as Gilgamesh's text has it
/// (CR 612.2).
fn equipment_card(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let equipment = q.words(choice.source).subtype(Subtype::Equipment);
    Verdict::each(choice, |card| q.chars(card).has_subtype(equipment))
}

/// "Look at the top six cards of your library. You may put any number of Equipment cards from among them onto the battlefield.
/// Put the rest on the bottom of your library in a random order. When you put one or more Equipment onto the battlefield this
/// way, ..." (CR 608.2c, 401.4, 603.12): the six cards are looked at (a smaller library shows what it has, CR 609.3), the
/// Equipment cards chosen enter together, and the reflexive ability triggers only if one or more did, about those that did.
fn look_at_top_six(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = select::cards(select::ANY_NUMBER).with(select::part(&(equipment_card as SelectFn), 0, "")).called("Equipment");
    let arrived = library::look_at_top_onto_battlefield_select(g, item.controller, item.object, 6, rule, false, library::Rest::BottomRandom)?;
    if !arrived.is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_about(item.object, item.controller, AbilityRef::delayed(this, 0), &arrived);
    }
    Ok(())
}

/// "You may attach one of them to a Samurai you control." (CR 701.3a-b, 603.12): one of the Equipment put onto the battlefield
/// this way that is still that object (CR 400.7), to a Samurai its controller controls as this resolves, Samurai read through
/// Gilgamesh's words (CR 612.2). It can't try to attach an Equipment to a creature it can't legally be attached to (the
/// ruling), so only Equipment with a Samurai to go to are offered, and then only those Samurai; choosing no Equipment is
/// declining.
fn may_attach_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let samurai_word = g.words(item.object).subtype(Subtype::Samurai);
    let samurai: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).has_subtype(samurai_word)).collect();
    let equipment: Vec<ObjectId> =
        g.event_objects(item).iter().copied().filter(|&e| samurai.iter().any(|&s| g.can_attach(e, s))).collect();
    let Some(&chosen) = choose::select(g, item.controller, item.object, Purpose::Attach, equipment, select::cards(select::up_to(1)).called("Equipment"))?.first() else {
        return Ok(());
    };
    let hosts: Vec<ObjectId> = samurai.into_iter().filter(|&s| g.can_attach(chosen, s)).collect();
    if let Some(&host) = choose::select(g, item.controller, item.object, Purpose::Choose, hosts, select::cards(select::exactly(1)).called("Samurai"))?.first() {
        g.attach(chosen, host);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        look_at_top_six,
        "Whenever Gilgamesh enters or attacks, look at the top six cards of your library. You may put any number of Equipment cards from among them onto the battlefield. Put the rest on the bottom of your library in a random order. When you put one or more Equipment onto the battlefield this way, you may attach one of them to a Samurai you control.",
    )],
    delayed: &[delayed_trigger(may_attach_one, "When you put one or more Equipment onto the battlefield this way, you may attach one of them to a Samurai you control.")],
    ..oracle::GILGAMESH_MASTER_AT_ARMS
};
