use cantrip_api::*;
use cantrip_mechanics::earthbend;
use cantrip_oracle as oracle;

/// "Whenever you tap a creature for mana" (CR 605.1b).
fn creature_tapped_for_mana(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::TappedForMana { permanent, player }
            if *player == q.controller(source) && q.chars(*permanent).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*permanent))
        }
        _ => None,
    }
}

fn add_green(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::G, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        earthbend::enters::<1>("When Badgermole Cub enters, earthbend 1."),
        triggered(when(EventMask::TAPPED, creature_tapped_for_mana), add_green, "Whenever you tap a creature for mana, add an additional {G}.")
            .mana_ability(),
    ],
    delayed: &[earthbend::RETURN],
    ..oracle::BADGERMOLE_CUB
};
