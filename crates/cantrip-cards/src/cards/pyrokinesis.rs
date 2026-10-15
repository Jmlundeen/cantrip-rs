use cantrip_api::*;
use cantrip_mechanics::{damage_divided, targets};
use cantrip_oracle as oracle;

fn red_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(Color::Red)
}

/// Pyrokinesis deals 4 damage divided as you choose among any number of target creatures (CR 601.2d): each target takes
/// its own share, and one that has since become illegal is left out with it.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef {
        targets: &[TargetSpec { min: 1, max: TargetSpec::COUNTED, filter: targets::CREATURE.filter }],
        effect,
    }),
    divide: Some(Divide { total: 4, of_x: None }),
    // You may exile a red card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[WayDef {
        text: "Exile a red card from your hand",
        cost: Cost { exile_from_hand: Some((1, red_card)), ..Cost::NONE },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }],
    ..oracle::PYROKINESIS
};
