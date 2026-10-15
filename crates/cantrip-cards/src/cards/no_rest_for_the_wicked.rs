use cantrip_api::*;
use cantrip_mechanics::history::put_into_graveyard_from_battlefield_this_turn;
use cantrip_oracle as oracle;

/// Return to your hand all creature cards in your graveyard that were put there from the battlefield this turn.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        put_into_graveyard_from_battlefield_this_turn(g, item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    g.return_to_hand(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "Sacrifice No Rest for the Wicked: Return to your hand all creature cards in your graveyard that were put there from the battlefield this turn.",
        cost: Cost { sacrifice_self: true, ..Cost::NONE },
        effect,
        ..ActivatedDef::BLANK
    })],
    ..oracle::NO_REST_FOR_THE_WICKED
};
