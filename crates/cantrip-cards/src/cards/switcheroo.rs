use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exchange control of two target creatures." (CR 701.12a-b): two different creatures (CR 601.2c), both still legal targets as it resolves, or the
/// exchange can't be completed and none of it happens (the card's ruling); two creatures one player controls exchange
/// nothing. Either may be anyone's (the ruling), and Auras and Equipment stay with their controllers.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let (Some(first), Some(second)) = (targets::object(g, item, 0), targets::object(g, item, 1)) {
        g.exchange_control(item.object, first, second);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE.between(2, 2)], effect), ..oracle::SWITCHEROO };
