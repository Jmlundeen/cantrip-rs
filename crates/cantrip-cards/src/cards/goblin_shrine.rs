use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

/// "When this Aura leaves the battlefield, it deals 1 damage to each Goblin creature." (CR 608.2, 120.3).
fn deals_1_damage_goblin_creature(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let recipients_0 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                q.chars(id).has_subtype(q.words(item.object).subtype(Subtype::Goblin))
                    && q.chars(id).is(CardType::Creature)
            })
            .collect::<Vec<ObjectId>>()
            .into_iter()
            .map(Target::Object)
            .collect::<Vec<Target>>()
    };
    let batch: Vec<(ObjectId, Target, u32)> = recipients_0
        .iter()
        .map(|t| (item.object, *t, 1))
        .collect();
    g.damage_together(&batch);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            cantrip_mechanics::triggers::THIS_LEAVES,
            deals_1_damage_goblin_creature,
            "When this Aura leaves the battlefield, it deals 1 damage to each Goblin creature.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::goblin_creatures_while_enchanted_land_is_basic_mountain,
            statics::plus_pt::<1, 0>,
            "As long as enchanted land is a basic Mountain, Goblin creatures get +1/+0.",
        ),
    ],
    enchant: Some(cantrip_mechanics::targets::LAND),
    ..oracle::GOBLIN_SHRINE
};
