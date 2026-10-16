use cantrip_api::*;
use cantrip_mechanics::evoke;
use cantrip_mechanics::hand::discard_chosen_from_revealed_hand;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

fn black_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::Black))
}

/// Target opponent reveals their hand; you choose a nonland card from it; they discard it. Asked even when nothing in
/// the hand is eligible: looking at the hand is part of the ability.
fn discard_nonland(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    discard_chosen_from_revealed_hand(g, item, opponent, Purpose::Discard, |q, c| !q.chars(c).is(CardType::Land))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, discard_nonland, "When this creature enters, target opponent reveals their hand. You choose a nonland card from it. That player discards that card.")
            .targets(&[targets::OPPONENT]),
        evoke::sacrifice_trigger("When Grief enters, if its evoke cost was paid, its controller sacrifices it."),
    ],
    // Evoke: exile a black card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(black_card), "Evoke\u{2014}Exile a black card from your hand")],
    ..oracle::GRIEF
};
