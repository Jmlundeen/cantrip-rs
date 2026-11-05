use cantrip_api::*;
use cantrip_mechanics::{choose, select, targets, triggers::ATTACHED_PERMANENT_BECOMES_TAPPED};
use cantrip_oracle as oracle;

/// "A land of their choice" the Aura can be attached to (CR 303.4d, 701.3a): a land on the battlefield `aura` may enchant.
fn land_this_aura_can_enchant(q: &dyn Query, choice: &Choice, aura: u32) -> Verdict {
    let aura = ObjectId(aura);
    Verdict::each(choice, |host| q.chars(host).is(CardType::Land) && q.can_attach_to(aura, host))
}

/// "When enchanted land becomes tapped, destroy it. That land's controller may attach this Aura to a land of their choice." (CR 608.2, 701.8a, 701.3a).
fn destroy_lands_controller_attach_aura(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = item.subject.object() else { return Ok(()) };
    g.destroy(land);
    g.end_sentence();
    let controller = g.controller(land);
    let lands = g.state().battlefield().to_vec();
    let rule = select::land_cards(select::up_to(1)).with(select::part(&(land_this_aura_can_enchant as SelectFn), item.object.0, ""));
    if let Some(&host) = choose::select(g, controller, item.object, Purpose::Choose, lands, rule)?.first() {
        instruction(g, |g| {
            g.attach(item.object, host);
        });
        g.end_sentence();
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            ATTACHED_PERMANENT_BECOMES_TAPPED,
            destroy_lands_controller_attach_aura,
            "When enchanted land becomes tapped, destroy it. That land's controller may attach this Aura to a land of their choice.",
        ),
    ],
    enchant: Some(targets::LAND),
    ..oracle::KUDZU
};
