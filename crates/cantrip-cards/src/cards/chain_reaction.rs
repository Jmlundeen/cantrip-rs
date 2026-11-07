use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Chain Reaction deals X damage to each creature, where X is the number of creatures on the battlefield." (CR 608.2, 120.3).
fn chain_reaction_deals_x_damage(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let amount_0 = {
        let q: &dyn Query = g;
        (q
            .state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).is(CardType::Creature))
            .collect::<Vec<ObjectId>>()
            .len() as i64) as u32
    };
    let recipients_0 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).is(CardType::Creature))
            .collect::<Vec<ObjectId>>()
            .into_iter()
            .map(Target::Object)
            .collect::<Vec<Target>>()
    };
    let batch: Vec<(ObjectId, Target, u32)> = recipients_0
        .iter()
        .map(|t| (item.object, *t, amount_0))
        .collect();
    g.damage_together(&batch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], chain_reaction_deals_x_damage),
    ..oracle::CHAIN_REACTION
};
