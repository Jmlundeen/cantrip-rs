use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_mechanics::triggers::AN_OPPONENT_DRAWS_A_CARD;
use cantrip_oracle as oracle;

/// "This creature deals 1 damage to them": the opponent who drew, the trigger's subject.
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.damage(item.object, item.subject, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(AN_OPPONENT_DRAWS_A_CARD, ping, "Whenever an opponent draws a card, this creature deals 1 damage to them.")],
    statics: &[static_ability(Layer::Ability, statics::itself_during_your_turn, statics::have_first_strike, "This creature has first strike during your turn.")],
    ..oracle::RAZORKIN_NEEDLEHEAD
};
