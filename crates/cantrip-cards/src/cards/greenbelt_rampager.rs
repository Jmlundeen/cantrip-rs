use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// Pay {E}{E} if able; otherwise return this permanent and get {E} (CR 118.3, 701.15).
fn pay_or_return(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if g.state().player(you).energy() >= 2 {
        g.change_energy(you, -2);
    } else {
        g.return_to_hand(&[item.object]);
        g.change_energy(you, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, pay_or_return,
        "When this creature enters, pay {E}{E}. If you can't, return this creature to its owner's hand and you get {E}.")],
    ..oracle::GREENBELT_RAMPAGER
};
