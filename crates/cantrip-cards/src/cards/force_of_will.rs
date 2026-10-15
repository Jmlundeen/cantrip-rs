use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

fn blue_card(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).colors.has(Color::Blue)
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    // You may pay 1 life and exile a blue card from your hand rather than pay this spell's mana cost (CR 118.9).
    ways: &[WayDef {
        text: "Pay 1 life, exile a blue card from your hand",
        cost: Cost { life: 1, exile_from_hand: Some((1, blue_card)), ..Cost::NONE },
        from: ZoneKind::Hand,
        exile_after: false,
        condition: None,
        instant_speed: false,
    }],
    ..oracle::FORCE_OF_WILL
};
