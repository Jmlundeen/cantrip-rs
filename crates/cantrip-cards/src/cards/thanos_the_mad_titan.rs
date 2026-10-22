use cantrip_api::*;
use cantrip_mechanics::{choose, effects, power_up};
use cantrip_oracle as oracle;

/// "Put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other creature with mana value of the chosen
/// quality. (... Zero is even.)" The choice is made as the ability resolves (CR 608.2d), and the creatures are destroyed
/// together (CR 701.8). "Other" leaves out this object alone: once Thanos has left, every creature of the quality is
/// destroyed.
fn snap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::two_plus_one_counters_on_this(g, item)?;
    let me = g.card_id(&CARD);
    let doomed: Vec<ObjectId> =
        choose::creatures_of_chosen_quality(g, item, me)?.into_iter().filter(|&id| id != item.object).collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    labels: choose::ODD_OR_EVEN,
    abilities: &[power_up::ability(
        cost("{C}{W}{U}{B}{R}{G}"),
        snap,
        "Power-up \u{2014} {C}{W}{U}{B}{R}{G}: Put two +1/+1 counters on Thanos. Choose odd or even. Destroy each other creature with mana value of the chosen quality. (Activate each power-up ability only once. Reduce the cost by his mana cost if he entered this turn. Zero is even.)",
    )],
    ..oracle::THANOS_THE_MAD_TITAN
};
