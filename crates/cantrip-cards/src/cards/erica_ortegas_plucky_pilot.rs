use cantrip_api::*;
use cantrip_mechanics::{choose, hand};
use cantrip_mechanics::select::{self, Selection};
use cantrip_oracle as oracle;

/// "Whenever Erica Ortegas becomes tapped" (CR 603.2e, 701.26a): any untapped-to-tapped event for this object.
fn erica_becomes_tapped(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Tapped { permanent } if *permanent == source => Fired::PLAIN,
        _ => None,
    }
}

/// "You may put a land card from your hand onto the battlefield. If you do" (CR 608.2c, 608.2d): only an actual entry
/// creates the delayed ability, controlled by the player who controlled this triggered ability as it resolved (CR 603.7e).
fn put_land_and_return_one_later(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(entered) = hand::may_put_onto_battlefield(g, item.controller, item.object, |c| c.is(CardType::Land))? else {
        return Ok(());
    };
    if !g.is_on_battlefield(entered) {
        return Ok(());
    }
    g.end_sentence();
    let card = g.card_id(&CARD);
    g.delayed_trigger(
        DelayedWhen::Step { step: Step::End, on_turn_of: None },
        item.object,
        item.controller,
        AbilityRef::delayed(card, 0),
        Target::None,
    );
    Ok(())
}

/// "At the beginning of the next end step, return a land you control to its owner's hand" (CR 603.7a, 603.7b, 608.2d).
/// The delayed trigger's controller chooses a land they control as it resolves; this is not a target.
fn return_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lands = g.permanents_of(item.controller).into_iter().filter(|&p| g.chars(p).is(CardType::Land)).collect();
    let rule = Selection::of(select::exactly(1), &[select::LAND]).called("land");
    let chosen = choose::select(g, item.controller, item.object, Purpose::ReturnToHand, lands, rule)?;
    g.return_to_hand(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::TAPPED, erica_becomes_tapped),
        put_land_and_return_one_later,
        "Whenever Erica Ortegas becomes tapped, you may put a land card from your hand onto the battlefield. If you do, return a land you control to its owner's hand at the beginning of the next end step.",
    )],
    delayed: &[delayed_trigger(
        return_a_land,
        "At the beginning of the next end step, return a land you control to its owner's hand.",
    )],
    ..oracle::ERICA_ORTEGAS_PLUCKY_PILOT
};
