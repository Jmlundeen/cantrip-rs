use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Chain Reaction deals X damage to each creature, where X is the number of creatures on the battlefield." (CR 120.3, 608.2c).
fn chain_reaction_deals_x_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = {
        let q: &dyn Query = g;
        q.state().battlefield().iter().filter(|&&id| q.chars(id).is(CardType::Creature)).count() as u32
    };
    effects::damage_each_creature(g, item, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], chain_reaction_deals_x_damage),
    ..oracle::CHAIN_REACTION
};
