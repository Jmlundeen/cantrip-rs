use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all enchantments." (CR 608.2, 701.8a).
fn destroy_all_enchantments(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let objects_0 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).is(CardType::Enchantment))
            .collect::<Vec<ObjectId>>()
    };
    g.destroy_all(&objects_0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_enchantments),
    ..oracle::TRANQUILITY
};
