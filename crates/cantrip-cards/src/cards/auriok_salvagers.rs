use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target artifact card with mana value 1 or less in your graveyard.
fn cheap_artifact_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    match t {
        Target::Object(id) => {
            targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact]) && q.card(q.chars(id).card).cost.mana_value() <= 1
        }
        _ => false,
    }
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.return_to_hand(&[card]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{W}"), effect, "{1}{W}: Return target artifact card with mana value 1 or less from your graveyard to your hand.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: cheap_artifact_in_your_graveyard, label: "" }])],
    ..oracle::AURIOK_SALVAGERS
};
