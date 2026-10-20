//! Harness (CR 701.64a) and infinity (CR 702.186): "[Cost]: Harness this permanent" gives it a designation that stays
//! with it, and the infinity ability is the permanent's only while it is harnessed. The Mind Stone prints both.
//!
//! A card with harness lists `ability(cost, text)` among its `abilities`, its infinity ability as its first `delayed`
//! ability, and `infinity(text)` among its `statics`.

use cantrip_api::*;

/// Whether the permanent is harnessed.
pub fn harnessed(q: &dyn LayerQuery, id: ObjectId) -> bool {
    q.state().effects.iter().any(|e| e.affected == id && e.kind == EffectKind::Harnessed)
}

/// It becomes harnessed, a designation that stays with the permanent. Harnessing a permanent that already is does nothing
/// (CR 701.64a).
fn harness(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let id = item.object;
    let already = g.state().effects.iter().any(|e| e.affected == id && e.kind == EffectKind::Harnessed);
    if !already && g.is_on_battlefield(id) && g.obj(id).became.is_none() {
        g.add_effect(id, id, EffectKind::Harnessed, Duration::Indefinite);
    }
    Ok(())
}

/// The ability that harnesses the permanent, for `cost`.
pub const fn ability(cost: Cost, text: &'static str) -> AbilityDef {
    cantrip_api::ability(cost, harness, text)
}

fn is_self_harnessed(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && harnessed(q, source)
}

/// The static that gives the infinity ability to the harnessed permanent.
pub const fn infinity(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, is_self_harnessed, crate::statics::grant_delayed_ability, text)
}
