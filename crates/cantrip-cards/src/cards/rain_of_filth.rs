use cantrip_api::*;
use cantrip_oracle as oracle;

/// Until end of turn, lands you control gain "Sacrifice this land: Add {B}." The set is fixed as this resolves
/// (CR 611.2c): a land that arrives later never gains the ability.
fn grant(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.obj(item.object).card;
    let lands: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).collect();
    instruction(g, |g| {
        for land in lands {
            g.add_effect(item.object, land, EffectKind::AddAbility(AbilityRef::delayed(this, 0)), Duration::EndOfTurn);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], grant),
    delayed: &[mana_ability(SACRIFICE_THIS, add(Mana::B), "Sacrifice this land: Add {B}.")],
    ..oracle::RAIN_OF_FILTH
};
