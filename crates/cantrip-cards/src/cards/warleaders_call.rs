use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures you control get +1/+1."
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn apply(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 1);
}

/// This enchantment deals 1 damage to each opponent.
fn strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g.state().opponents(item.controller).map(|p| (item.object, Target::Player(p), 1)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, affects, apply, "Creatures you control get +1/+1.")],
    abilities: &[triggered(cantrip_mechanics::triggers::CREATURE_YOU_CONTROL_ENTERS, strike, "Whenever a creature you control enters, this enchantment deals 1 damage to each opponent.")],
    ..oracle::WARLEADERS_CALL
};
