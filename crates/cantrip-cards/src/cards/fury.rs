use cantrip_api::*;
use cantrip_mechanics::evoke;
use cantrip_mechanics::{damage_divided, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

fn red_card(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(q.words(asker.source).color(Color::Red))
}

/// Fury deals 4 damage divided as you choose among any number of target creatures and/or planeswalkers (CR 601.2d):
/// each target takes its own share, and one that has since become illegal is left out with it.
fn divided_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, divided_damage, "When this creature enters, it deals 4 damage divided as you choose among any number of target creatures and/or planeswalkers.")
            .targets(&[TargetSpec { min: 1, max: TargetSpec::COUNTED, filter: targets::CREATURE_OR_PLANESWALKER.filter, label: "" }])
            .divide(Divide { total: 4, of_x: None }),
        evoke::sacrifice_trigger("When Fury enters, if its evoke cost was paid, its controller sacrifices it."),
    ],
    // Evoke: exile a red card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[evoke::way(evoke::exile_from_hand(red_card), "Evoke\u{2014}Exile a red card from your hand")],
    ..oracle::FURY
};
