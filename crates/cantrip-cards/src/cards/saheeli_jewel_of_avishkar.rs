use cantrip_api::*;
use cantrip_mechanics::statics::have_haste;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "Thopters you control have haste" (CR 613.1f, layer 6, 702.10a): every Thopter under the source's controller's
/// control as the layers compute it, whoever owns it, and after the earlier layers have made it one.
fn thopters_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    so_far.has_subtype(Subtype::Thopter) && s.obj(affected).controller == s.obj(source).controller
}

/// "Whenever you cast a noncreature spell, create a 1/1 colorless Thopter artifact creature token with flying."
fn make_a_thopter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::THOPTER, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::NONCREATURE_SPELL_YOU_CAST,
        make_a_thopter,
        "Whenever you cast a noncreature spell, create a 1/1 colorless Thopter artifact creature token with flying.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        thopters_you_control,
        have_haste,
        "Thopters you control have haste.",
    )],
    ..oracle::SAHEELI_JEWEL_OF_AVISHKAR
};

pub static EXTRA: &[&CardDef] = &[&tokens::THOPTER];
