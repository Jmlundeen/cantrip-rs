use cantrip_api::*;
use cantrip_mechanics::{ninjutsu, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters from your hand, if it's attacking" (CR 603.2, 603.4, 603.6a): its own entering from its
/// controller's hand, while it is attacking (CR 508.4: ninjutsu puts it onto the battlefield attacking).
fn when_creature_enters_hand(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::this_enters(q, asker.source, event)?;
    let Event::ZoneChange { from, .. } = event else { return None };
    (*from == Zone::hand(asker.controller) && q.is_attacking(asker.source)).then_some(fired)
}

/// "When this creature enters from your hand, if it's attacking, create a token that's a copy of another target attacking
/// creature. The token enters tapped and attacking." The intervening "if" is rechecked as the ability resolves, from last-known
/// information if the Shadow has left (CR 603.4); the token is a copy of the target (CR 707.2) and its controller chooses what it
/// attacks as it enters (CR 508.4).
fn if_attacking_create_token_thats(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.is_attacking(item.object) && !g.was_attacking(item.object) {
        return Ok(());
    }
    let Some(original) = targets::object(g, item, 0) else { return Ok(()) };
    g.create_token_copies_attacking(g.copiable_values(original), item.controller, 1, true, None)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ninjutsu::ninjutsu(mana("{2}{U}{U}"), "Ninjutsu {2}{U}{U}"),
        triggered(
                when_asked(EventMask::ZONE_CHANGE, when_creature_enters_hand),
                if_attacking_create_token_thats,
                "When this creature enters from your hand, if it's attacking, create a token that's a copy of another target attacking creature. The token enters tapped and attacking.",
            )
            .targets(&[targets::ANOTHER_ATTACKING_CREATURE]),
    ],
    ..oracle::THOUSAND_FACED_SHADOW
};
