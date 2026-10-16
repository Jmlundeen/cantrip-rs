use cantrip_api::*;
use cantrip_mechanics::destroy_target;
use cantrip_oracle as oracle;

fn nonbasic_land(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && {
        let chars = q.chars(id);
        chars.is(CardType::Land) && !chars.supertypes.basic
    })
}

/// Destroy target nonbasic land.
fn destroy_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(TAP.and(SACRIFICE_THIS), destroy_land, "{T}, Sacrifice Wasteland: Destroy target nonbasic land.")
            .targets(&[target(nonbasic_land)]),
    ],
    ..oracle::WASTELAND
};
