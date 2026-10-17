use cantrip_api::*;
use cantrip_mechanics::{destroy_target, outlaw};
use cantrip_oracle as oracle;

/// "Target non-outlaw creature" (CR 115.1, 700.12).
fn non_outlaw_creature(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !outlaw::is_outlaw(&c)
    })
}

/// "Destroy target non-outlaw creature." (CR 701.8)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(non_outlaw_creature)], effect),
    ..oracle::SHOOT_THE_SHERIFF
};
