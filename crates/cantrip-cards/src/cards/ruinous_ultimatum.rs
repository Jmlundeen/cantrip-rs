use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all nonland permanents your opponents control." (CR 701.8a, 109.5): every nonland permanent each
/// opponent controls is destroyed at once, in one event. Your own permanents are left alone, and so is every land.
fn destroy_all_nonland_permanents_your_opponents_control(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let victims: Vec<ObjectId> = g
        .state()
        .opponents(you)
        .flat_map(|player| g.permanents_of(player))
        .filter(|&id| !g.chars(id).is(CardType::Land))
        .collect();
    g.destroy_all(&victims);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_nonland_permanents_your_opponents_control),
    ..oracle::RUINOUS_ULTIMATUM
};
