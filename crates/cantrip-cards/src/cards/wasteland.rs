use cantrip_api::*;
use cantrip_mechanics::{destroy_target, mana};
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
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice Wasteland: Destroy target nonbasic land.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            targets: &[TargetSpec { min: 1, max: 1, filter: nonbasic_land }],
            effect: destroy_land,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::WASTELAND
};
