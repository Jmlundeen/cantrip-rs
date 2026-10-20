use cantrip_api::*;
use cantrip_mechanics::{channel, choose};
use cantrip_oracle as oracle;

/// "Mill three cards, then return a creature or planeswalker card from your graveyard to your hand": chosen from the
/// whole graveyard as it stands after the mill, and not optional - one is returned whenever there is one.
fn mill_then_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.mill(you, 3);
    let graveyard = g.state().zone(Zone::graveyard(you)).to_vec();
    let eligible: Vec<ObjectId> = graveyard
        .iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
        })
        .collect();
    if let Some(chosen) = choose::cards_from(g, you, item.object, Purpose::PutIntoHand, graveyard, eligible, 1, false)?.first() {
        g.return_to_hand(&[*chosen]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        ability(cost("{3}{B}").and(DISCARD_THIS), mill_then_return, "Channel \u{2014} {3}{B}, Discard this card: Mill three cards, then return a creature or planeswalker card from your graveyard to your hand. This ability costs {1} less to activate for each legendary creature you control.")
            .from(ZoneKind::Hand),
    ],
    own_cost_changes: &[channel::cheaper_per_legendary_creature],
    ..oracle::TAKENUMA_ABANDONED_MIRE
};
