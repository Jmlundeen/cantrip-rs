use cantrip_api::*;
use cantrip_mechanics::effects::this_gains_first_strike;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// "Exile this creature. Return it to the battlefield under its owner's control at the beginning of the next end step"
/// (CR 701.13, 603.7): the delayed ability is created as this resolves whether or not anything was exiled (CR 603.7a), and
/// is about the object exiled (CR 603.7c): if this had already left the battlefield it affects nothing, and it affects
/// nothing if the card left exile in between (CR 400.7).
fn blink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let exiled = g.exile_all(&[item.object]);
    delayed::at_next_end_step_about(g, item, 0, &exiled);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{W}").and(exile_from_graveyard(3)), this_gains_first_strike, "{W}, Exile three cards from your graveyard: This creature gains first strike until end of turn."),
        ability(discard_cards(1), blink, "Discard a card: Exile this creature. Return it to the battlefield under its owner's control at the beginning of the next end step."),
    ],
    delayed: &[delayed_trigger(delayed::return_exiled_to_battlefield, "Return it to the battlefield under its owner's control.")],
    ..oracle::FLEETING_SPIRIT
};
