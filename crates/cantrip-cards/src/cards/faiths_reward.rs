use cantrip_api::*;
use cantrip_mechanics::history::put_into_graveyard_from_battlefield_this_turn;
use cantrip_oracle as oracle;

/// Return to the battlefield all permanent cards in your graveyard that were put there from the battlefield this turn
/// (CR 400.7: the cards the moves made, still there), all at once under your control.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let back: Vec<(ObjectId, PlayerId)> = put_into_graveyard_from_battlefield_this_turn(g, item.controller)
        .into_iter()
        .filter(|&id| {
            let c = g.chars(id);
            [CardType::Artifact, CardType::Creature, CardType::Enchantment, CardType::Land, CardType::Planeswalker, CardType::Battle]
                .iter()
                .any(|&t| c.is(t))
        })
        .map(|id| (id, item.controller))
        .collect();
    if !back.is_empty() {
        g.put_all_onto_battlefield(&back)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::FAITHS_REWARD };
