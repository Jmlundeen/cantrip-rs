use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Legendary creatures don't untap during their controllers' untap steps. When this artifact enters, tap all legendary creatures." (CR 604.1).
fn legendary_creatures_dont_untap_during_rule(
    q: &dyn Query,
    _: &Asker,
    id: ObjectId,
) -> bool {
    q.chars(id).supertypes.legendary && q.chars(id).is(CardType::Creature)
}

/// "When this artifact enters, tap all legendary creatures." (CR 608.2, 701.26a).
fn tap_all_legendary_creatures(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let objects_0 = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                q.chars(id).supertypes.legendary && q.chars(id).is(CardType::Creature)
            })
            .collect::<Vec<ObjectId>>()
    };
    g.tap_together(&objects_0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            tap_all_legendary_creatures,
            "When this artifact enters, tap all legendary creatures.",
        ),
    ],
    untap_bans: &[legendary_creatures_dont_untap_during_rule],
    ..oracle::ARENA_OF_THE_ANCIENTS
};
