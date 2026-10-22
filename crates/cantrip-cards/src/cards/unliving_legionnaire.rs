use cantrip_api::*;
use cantrip_mechanics::{effects, power_up, targets};
use cantrip_oracle as oracle;

/// "Return up to one target creature card from your graveyard to your hand. Put two +1/+1 counters on this creature."
/// With no target chosen it resolves and puts the counters on; with a target that is gone as it would resolve, it does
/// not resolve at all (CR 608.2b), which the core decides.
fn return_and_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    effects::two_plus_one_counters_on_this(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[power_up::ability(
        cost("{5}{B}{B}"),
        return_and_counters,
        "Power-up \u{2014} {5}{B}{B}: Return up to one target creature card from your graveyard to your hand. Put two +1/+1 counters on this creature. (Activate each power-up ability only once. Reduce the cost by its mana cost if it entered this turn.)",
    )
    .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.optional()])],
    ..oracle::UNLIVING_LEGIONNAIRE
};
